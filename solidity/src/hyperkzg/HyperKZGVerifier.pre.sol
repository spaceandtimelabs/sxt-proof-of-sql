// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title HyperKZGHelpers
/// @notice Library providing the HyperKZG polynomial commitment verifier.
/// @notice See section 6 of the [MicroNova paper](https://eprint.iacr.org/2024/2099.pdf#section.6)
/// for details on the HyperKZG protocol.
///
/// ## Notation
/// Sections 6.1 and 6.2 of the paper use somewhat different notation that is furthermore, different from the code.
/// We have tried to be fairly consistent with the reference implementation, as well as the paper.
/// The following is a mapping to resolve any confusion.
///
/// \\[ \begin{aligned}
/// \texttt{com[i]}&=\overline{P}^{(i+1)}=C_{i+2} & i&\in[0,\ell-1)\\\\
/// \texttt{v[i][0]}&=v_{i+1,1}=y_{pos}^{(i)} & i&\in[0,\ell)\\\\
/// \texttt{v[i][1]}&=v_{i+1,2}=y_{neg}^{(i)} & i&\in[0,\ell)\\\\
/// \texttt{v[i][2]}&=v_{i+1,3}=y^{(i)} & i&\in[0,\ell)\\\\
/// \texttt{w[j]}&=W_{j+1} & j&\in[0,3)\\\\
/// \texttt{r}&=r \text{ and } (u_1,u_2,u_3)=(r,-r,r^2)\\\\
/// \texttt{q}&=q\\\\
/// \texttt{d}&=d=d_2 \text{ and } d_3=d^2\\\\
/// \texttt{y}&=y^{(\ell)}=v_{\ell+1,3}\\\\
/// \texttt{commitment}&=\overline{P}=\overline{P}^{(0)}=C_1\\\\
/// \texttt{x[i]}&=x_{i+1} & i&\in[0,\ell)
/// \end{aligned} \\]
///
/// ## Formulas and Checks
///
/// Staying closely aligned with the notation in the paper, we define
/// $$\begin{aligned}
/// C_B &= \sum_{i=1}^\ell q^{i-1}C_i\\\\
/// B(u_j) &= \sum_{i=1}^\ell q^{i-1}v_{i,j}\\\\
/// L_j &= C_B - B(u_j) G + u_j W_j\\\\
/// R_j &= W_j\\\\
/// L &= L_1+d_2L_2+d_3L_3\\\\
/// R &= R_1+d_2R_2+d_3R_3
/// \end{aligned}$$
///
/// We then check the following:
/// $$\begin{aligned}
/// y^{(i)} &= (1-x_i)\cdot\frac{y^{(i-1)}\_{pos}+y^{(i-1)}\_{neg}}{2}
/// +x_i\cdot\frac{y^{(i-1)}\_{pos}-y^{(i-1)}\_{neg}}{2\cdot r}&i\in[1,\ell]\\\\
/// e(L,H)&=e(R,\tau H)
/// \end{aligned}$$
/// NOTE: the paper has some typos in the first formula (which will likely be corrected soon).
/// In particular, \\( i=\ell \\) is not checked, but needs to be.
///
/// ## Simplification
///
/// We don't do the computation directly as above.
/// Instead, we combine the formulas to simplify the computation as follows:
/// $$\begin{aligned}
/// b &= \sum_{i=0}^{\ell-1}\sum_{j=0}^2 q^id^j v_{i+1,j+1} \\\\
/// L &= (1+d+d^2)\cdot \left(C_1+q \cdot \sum_{i=0}^{\ell-2}q^i\cdot C_{i+2}\right) \\\\
/// &\phantom{=} + b\cdot\left(-G\right) +r\cdot W_1+(-dr)\cdot W_2+(dr)^2\cdot W_3\\\\
/// R &= \sum_{j=0}^2 d^2\cdot W_{j+1}
/// \end{aligned}$$
/// We rewrite the checks as:
/// $$\begin{aligned}
/// 0&=r \cdot (2v_{i+2,3} + (x_{i+1} - 1) \cdot (v_{i+1,2} + v_{i+1,1}))\\\\
/// &\phantom{=} + x_{i+1} \cdot (v_{i+1,2} - v_{i+1,1})&i\in[0,\ell)\\\\
/// 0&=e(L,-H)+e(R,\tau H)
/// \end{aligned}$$
///
/// ## Array formulas
///
/// The actual code indexes starting with 0. Using arrays, we write the equations above to align closely with the code:
/// $$\begin{aligned}
/// b &= \sum_{i=0}^{\ell-1}\sum_{j=0}^2 q^id^j \texttt{v}[i][j] \\\\
/// L &= (1+d+d^2)\cdot \left(\texttt{commitment}+q \cdot \sum_{i=0}^{\ell-2}q^i\cdot \texttt{com}[i]\right) \\\\
/// &\phantom{=} + b\cdot\left(-G\right) \\\\
/// &\phantom{=} +r\cdot \texttt{w}[0]+(-dr)\cdot \texttt{w}[1]+(dr)^2\cdot \texttt{w}[2]\\\\
/// R &= \sum_{j=0}^2 d^j\cdot \texttt{w}[j]
/// \end{aligned}$$
/// and the checks are:
/// $$\begin{aligned}
/// 0&=r \cdot (2\cdot\texttt{v}[i+1][2] + (\texttt{x}[i] - 1) \cdot (\texttt{v}[i][1] + \texttt{v}[i][0])) \\\\
/// &\phantom{=} + \texttt{x}[i] \cdot (\texttt{v}[i][1] - \texttt{v}[i][0])&i\in[0,\ell)\\\\
/// 0&=e(L,-H)+e(R,\tau H)
/// \end{aligned}$$
///
/// Note that \\( \texttt{v}[\ell][2]=\texttt{y} \\).
library HyperKZGVerifier {
    /// @notice Verify a HyperKZG proof for polynomial commitment
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// verify_hyperkzg(proof_ptr, transcript_ptr, commitment_ptr, x, y)
    /// ```
    /// ##### Parameters
    /// * `proof_ptr` - the calldata pointer to the beginning of the proof data
    /// * `transcript_ptr` - the memory pointer to the transcript word
    /// * `commitment_ptr` - the memory pointer to the commitment point
    /// * `x` - the memory pointer to the array of x coordinates
    /// * `y` - the y value being verified
    /// @dev This function verifies a HyperKZG proof by:*
    /// 1. Running a Fiat-Shamir transcript to generate challenges r, q, d
    /// WARNING: The public inputs (x, y, the commitments, digest of the KZG SRS, degree bound, etc) are
    /// NOT included in the transcript and need to be added, either explicitly or implicitly,
    /// before calling this function
    /// 2. Computing the bivariate evaluation b:
    ///    \\[ b = \sum_{i=0}^{\ell-1}\sum_{j=0}^2 q^id^j \texttt{v}[i][j] \\]
    /// 3. Verifying v array consistency with the evaluation points by checking for each i:
    ///    \\[ r \cdot (2\cdot\texttt{v}[i+1][2] + (\texttt{x}[i] - 1) \cdot (\texttt{v}[i][1] + \texttt{v}[i][0])) +
    ///          \texttt{x}[i] \cdot (\texttt{v}[i][1] - \texttt{v}[i][0]) = 0 \\]
    ///    where \\( \texttt{v}[\ell][2] = y \\)
    /// 4. Computing the left group element using multi-scalar multiplication:
    ///    \\[ L = (1+d+d^2)\cdot \left(\texttt{commitment}+q \cdot \sum_{i=0}^{\ell-2}q^i\cdot \texttt{com}[i]\right) +
    ///        b\cdot\left(-G\right)+r\cdot \texttt{w}[0]+(-dr)\cdot \texttt{w}[1]+(dr)^2\cdot \texttt{w}[2] \\]
    /// 5. Computing the right group element:
    ///    \\[ R = \sum_{j=0}^2 d^2\cdot \texttt{w}[j] \\]
    /// 6. Verifying the pairing equation:
    ///    \\[ 0=e(L,-H)+e(R,\tau H)\\]
    /// @param __proof The HyperKZG proof data containing commitments, v values, and witness points
    /// @param __transcript Initial transcript value
    /// @param __commitment The polynomial commitment point C
    /// @param __x Array of x coordinates for evaluation
    /// @param __y The claimed evaluation result
    function __verifyHyperKZG( // solhint-disable-line gas-calldata-parameters
        bytes calldata __proof,
        uint256[1] memory __transcript,
        uint256[2] memory __commitment,
        uint256[] memory __x,
        uint256 __y
    ) external view {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenge(transcript_ptr) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_add(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_mul(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_pairing_x2(args_ptr) -> success {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_mul_assign(args_ptr, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function calldata_ec_mul_add_assign(args_ptr, c_ptr, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_add_assign(args_ptr, c_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function constant_ec_mul_add_assign(args_ptr, x, y, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function calldata_ec_add_assign(args_ptr, c_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL HyperKZGHelpers.pre.sol
            function run_transcript(com_ptr, v_ptr, w_ptr, transcript_ptr, ell) -> r, q, d {
                revert(0, 0)
            }
            // IMPORT-YUL HyperKZGHelpers.pre.sol
            function bivariate_evaluation(v_ptr, q, d, ell) -> b {
                revert(0, 0)
            }
            // IMPORT-YUL HyperKZGHelpers.pre.sol
            function check_v_consistency(v_ptr, r, x, y) {
                revert(0, 0)
            }
            // IMPORT-YUL HyperKZGHelpers.pre.sol
            function univariate_group_evaluation(g_ptr, e, length, scratch) {
                revert(0, 0)
            }
            // IMPORT-YUL HyperKZGHelpers.pre.sol
            function compute_gl_msm(com_ptr, length, w_ptr, commitment_ptr, r, q, d, b, scratch) {
                revert(0, 0)
            }

            function verify_hyperkzg(proof_ptr, transcript_ptr, commitment_ptr, x, y) {
                function v_ptr(ptr, l) -> result {
                    result := add(ptr, sub(mul(WORDX2_SIZE, l), WORDX2_SIZE))
                }
                function w_ptr(ptr, l) -> result {
                    result := add(ptr, sub(mul(WORDX5_SIZE, l), WORDX2_SIZE))
                }

                let ell := mload(x)

                // if ell == 0, then error
                if iszero(ell) { err(ERR_HYPER_KZG_EMPTY_POINT) }

                // Step 1: Run the transcript
                // WARNING: The public inputs (x, y, the commitments, digest of the KZG SRS, degree bound, etc) are
                // NOT included in the transcript and need to be added, either explicitly or implicitly,
                // before calling this function
                let r, q, d :=
                    run_transcript(proof_ptr, v_ptr(proof_ptr, ell), w_ptr(proof_ptr, ell), transcript_ptr, ell)

                // Step 2: Compute bivariate evaluation
                let b := bivariate_evaluation(v_ptr(proof_ptr, ell), q, d, ell)

                // Step 3: Check v consistency
                check_v_consistency(v_ptr(proof_ptr, ell), r, x, y)

                // Allocate scratch space for L, R, and the pairing check
                let scratch := mload(FREE_PTR)

                // Step 4: Compute L
                compute_gl_msm(proof_ptr, sub(ell, 1), w_ptr(proof_ptr, ell), commitment_ptr, r, q, d, b, scratch)

                // Step 5: Compute R
                univariate_group_evaluation(w_ptr(proof_ptr, ell), d, 3, add(scratch, WORDX6_SIZE))

                // Step 6: Verify the pairing equation
                mstore(add(scratch, WORDX2_SIZE), G2_NEG_GEN_X_IMAG)
                mstore(add(scratch, WORDX3_SIZE), G2_NEG_GEN_X_REAL)
                mstore(add(scratch, WORDX4_SIZE), G2_NEG_GEN_Y_IMAG)
                mstore(add(scratch, WORDX5_SIZE), G2_NEG_GEN_Y_REAL)
                mstore(add(scratch, WORDX8_SIZE), VK_TAU_HX_IMAG)
                mstore(add(scratch, WORDX9_SIZE), VK_TAU_HX_REAL)
                mstore(add(scratch, WORDX10_SIZE), VK_TAU_HY_IMAG)
                mstore(add(scratch, WORDX11_SIZE), VK_TAU_HY_REAL)
                if iszero(ec_pairing_x2(scratch)) { err(ERR_HYPER_KZG_PAIRING_CHECK_FAILED) }
            }
            verify_hyperkzg(__proof.offset, __transcript, __commitment, __x, __y)
        }
    }
}
