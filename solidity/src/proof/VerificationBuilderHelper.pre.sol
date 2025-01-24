// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol"; // solhint-disable-line no-global-import

library Helper {
    function wrapper() public pure {
        assembly {
            // IMPORT-YUL VerificationBuilder.sol
            function set_one_evaluations(builder_ptr, one_evaluation_ptr, one_evaluation_length) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.sol
            function compute_truncated_lagrange_basis_sum(length, point_ptr, num_vars) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_word(ptr) -> ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_words(ptr, count) -> ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_u64(ptr) -> ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function calldataload_u64(i) -> value {
                revert(0, 0)
            }
            function allocate_and_set_one_evaluation_lengths(builder_ptr, _proof_ptr) -> proof_ptr {
                proof_ptr := _proof_ptr

                // Read number of one evaluations from proof
                let num_indicators := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)

                // Allocate space for one evaluations
                let one_evaluation_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, increment_words(one_evaluation_ptr, num_indicators))

                // Set one evaluations in builder
                set_one_evaluations(builder_ptr, one_evaluation_ptr, num_indicators)

                // Read one evaluations lengths from proof
                for {} num_indicators { num_indicators := sub(num_indicators, 1) } {
                    mstore(one_evaluation_ptr, calldataload_u64(proof_ptr))
                    one_evaluation_ptr := increment_word(one_evaluation_ptr)
                    proof_ptr := increment_u64(proof_ptr)
                }
            }
            function compute_one_evaluations(builder_ptr, point_ptr, num_vars) {
                let head_ptr := mload(add(builder_ptr, ONE_EVALUATION_HEAD_OFFSET))
                let tail_ptr := mload(add(builder_ptr, ONE_EVALUATION_TAIL_OFFSET))
                // for (; head_ptr <= tail_ptr; ++head_ptr)
                for {} iszero(gt(head_ptr, tail_ptr)) { head_ptr := increment_word(head_ptr) } {
                    mstore(head_ptr, compute_truncated_lagrange_basis_sum(mload(head_ptr), point_ptr, num_vars))
                }
            }
        }
    }
}
