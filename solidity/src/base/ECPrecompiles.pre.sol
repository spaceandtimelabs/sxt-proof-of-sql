// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

// assembly only constants
/* solhint-disable no-unused-import */
import {
    ECADD_ADDRESS,
    ECADD_GAS,
    ECMUL_ADDRESS,
    ECMUL_GAS,
    ECPAIRING_ADDRESS,
    ECPAIRINGX2_GAS,
    INVALID_EC_ADD_INPUTS,
    INVALID_EC_MUL_INPUTS,
    INVALID_EC_PAIRING_INPUTS,
    WORD_SIZE,
    WORDX2_SIZE,
    WORDX3_SIZE,
    WORDX4_SIZE,
    WORDX12_SIZE
} from "./Constants.sol";
/* solhint-enable no-unused-import */

/// @title ECPrecompiles Library
/// @notice A library holding Yul wrappers for the precompiled contracts.
library ECPrecompiles {
    /// @notice Wrapper around the ECADD precompile.
    /// @dev The words are in the format [a_x, a_y, b_x, b_y], where the point a = (a_x, a_y) and b = (b_x, b_y).
    /// This function does an in-place addition of the points a and b. In other words, it sets a += b.
    /// The result is stored in the first two words of the input. If c = a + b, then the input memory is
    /// modified to be [c_x, c_y, b_x, b_y].
    /// @param argsPtr0 The input memory containing the points to be added.
    function ecAdd(uint256[4] memory argsPtr0) internal view {
        assembly {
            function ec_add(args_ptr) {
                if iszero(staticcall(ECADD_GAS, ECADD_ADDRESS, args_ptr, WORDX4_SIZE, args_ptr, WORDX2_SIZE)) {
                    mstore(0, INVALID_EC_ADD_INPUTS)
                    revert(0, 4)
                }
            }
            ec_add(argsPtr0)
        }
    }

    /// @notice Wrapper around the ECMUL precompile.
    /// @dev The words are in the format [a_x, a_y, scalar], where the point a = (a_x, a_y) and scalar is the scalar to multiply by.
    /// This function does an in-place multiplication of the point a by the scalar. In other words, it sets a *= scalar.
    /// The result is stored in the first two words of the input. If c = a * scalar, then the input memory is
    /// modified to be [c_x, c_y, scalar].
    /// @param argsPtr0 The input memory containing the point and scalar to be multiplied.
    function ecMul(uint256[3] memory argsPtr0) internal view {
        assembly {
            function ec_mul(args_ptr) {
                if iszero(staticcall(ECMUL_GAS, ECMUL_ADDRESS, args_ptr, WORDX3_SIZE, args_ptr, WORDX2_SIZE)) {
                    mstore(0, INVALID_EC_MUL_INPUTS)
                    revert(0, 4)
                }
            }
            ec_mul(argsPtr0)
        }
    }

    /// @notice Wrapper around the ECPAIRING precompile.
    /// @dev The words are in the format [a_x, a_y, g_x_imag, g_x_real, g_y_imag, g_y_real, b_x, b_y, h_x_imag, h_x_real, h_y_imag, h_y_real].
    /// Where the point a = (a_x, a_y) and the points g = (g_x_real + g_x_imag * i, g_y_real + g_y_imag * i),
    /// b = (b_x, b_y), and h = (h_x_real + h_x_imag * i, h_y_real + h_y_imag * i).
    /// This function computes the pairing check e(a, b) + e(g, h) == 0.
    /// If the pairing check is successful, the function returns 1. Otherwise, it returns 0.
    /// The input memory will have the first slot replaced by the returned value.
    /// @param argsPtr0 The input memory containing the points for the pairing check.
    /// @return success0 The result of the pairing check.
    function ecPairingX2(uint256[12] memory argsPtr0) internal view returns (uint256 success0) {
        assembly {
            function ec_pairing_x2(args_ptr) -> success {
                if iszero(staticcall(ECPAIRINGX2_GAS, ECPAIRING_ADDRESS, args_ptr, WORDX12_SIZE, args_ptr, WORD_SIZE)) {
                    mstore(0, INVALID_EC_PAIRING_INPUTS)
                    revert(0, 4)
                }
                success := mload(args_ptr)
            }
            success0 := ec_pairing_x2(argsPtr0)
        }
    }

    /// @notice Convenience function for multiplying a point by a scalar in place.
    /// @dev This is a thin wrapper around `ecMul` that sets the scalar in the input memory.
    /// In effect, this function does the operation `a *= scalar`.
    /// The input memory is in the format [a_x, a_y, _]. The third slot is used as scratch space to store the scalar.
    /// @param argsPtr0 The input memory containing the point to be multiplied.
    /// @param scalar0 The scalar to multiply the point by.
    function ecMulAssign(uint256[3] memory argsPtr0, uint256 scalar0) internal view {
        assembly {
            // IMPORT-YUL ECPrecompiles.pre.sol
            function ec_mul(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            function ec_mul_assign(args_ptr, scalar) {
                mstore(add(args_ptr, WORDX2_SIZE), scalar)
                ec_mul(args_ptr)
            }
            ec_mul_assign(argsPtr0, scalar0)
        }
    }

    /// @notice Wrapper around the ECADD precompile that takes the second point as calldata.
    /// @dev The first point is in memory, and the second point is in calldata.
    /// In effect, this function does the operation `a += c`, where c is in calldata and a is in memory.
    /// The input memory is in the format [a_x, a_y, _, _]. The third and fourth slots are used as scratch space.
    /// @param argsPtr0 The input memory containing the first point.
    /// @param cPtr0 The calldata containing the second point.
    /// @return argsPtrOut0 The result of the addition.
    function calldataECAddAssign( // solhint-disable-line gas-calldata-parameters
    uint256[4] memory argsPtr0, uint256[2] calldata cPtr0)
        external
        view
        returns (uint256[4] memory argsPtrOut0)
    {
        assembly {
            // IMPORT-YUL ECPrecompiles.pre.sol
            function ec_add(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            function calldata_ec_add_assign(args_ptr, c_ptr) {
                calldatacopy(add(args_ptr, WORDX2_SIZE), c_ptr, WORDX2_SIZE)
                ec_add(args_ptr)
            }
            calldata_ec_add_assign(argsPtr0, cPtr0)
        }
        argsPtrOut0 = argsPtr0;
    }

    /// @notice Convenience function for multiplying a point by a scalar and adding another point in place.
    /// @dev In effect, this function does the operation `a += c * scalar`.
    /// The first point is in memory, the second point is in calldata, and the scalar is in the stack.
    /// The input memory is in the format [a_x, a_y, _, _, _]. The third and fourth slots are used as scratch space.
    /// @param argsPtr0 The input memory containing the first point.
    /// @param cPtr0 The calldata containing the second point.
    /// @param scalar0 The scalar to multiply the second point by.
    /// @return argsPtrOut0 The result of the operation.
    function calldataECMulAddAssign( // solhint-disable-line gas-calldata-parameters
    uint256[5] memory argsPtr0, uint256[2] calldata cPtr0, uint256 scalar0)
        external
        view
        returns (uint256[5] memory argsPtrOut0)
    {
        assembly {
            // LCOV_EXCL_START
            // IMPORT-YUL ECPrecompiles.pre.sol
            function ec_add(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ECPrecompiles.pre.sol
            function ec_mul(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ECPrecompiles.pre.sol
            function ec_mul_assign(args_ptr, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // LCOV_EXCL_STOP
            function calldata_ec_mul_add_assign(args_ptr, c_ptr, scalar) {
                calldatacopy(add(args_ptr, WORDX2_SIZE), c_ptr, WORDX2_SIZE)
                ec_mul_assign(add(args_ptr, WORDX2_SIZE), scalar)
                ec_add(args_ptr)
            }
            calldata_ec_mul_add_assign(argsPtr0, cPtr0, scalar0)
        }
        argsPtrOut0 = argsPtr0;
    }
}
