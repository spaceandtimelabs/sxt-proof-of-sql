// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";

/// @title Plan Utility Library
/// @notice A library for handling utility functions related to plans.
library PlanUtil {
    /// @notice Skips over the names in a plan and returns the updated pointer.
    /// @notice This is a wrapper around the `skip_plan_names` Yul function.
    /// This wrapper is only intended to be used for testing.
    /// @param __plan The calldata pointer to the plan.
    /// @return __planOut The updated pointer after skipping names.
    function __skipPlanNames(bytes calldata __plan) internal pure returns (bytes calldata __planOut) {
        assembly {
            function skip_plan_names(plan_ptr) -> plan_ptr_out {
                // skip over the table names
                let num_tables := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_tables { num_tables := sub(num_tables, 1) } {
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                }
                // skip over the column names
                let num_columns := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_columns { num_columns := sub(num_columns, 1) } {
                    plan_ptr := add(plan_ptr, UINT64_SIZE)
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                    plan_ptr := add(plan_ptr, UINT32_SIZE)
                }
                // skip over the output column names
                let num_outputs := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_outputs { num_outputs := sub(num_outputs, 1) } {
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                }

                plan_ptr_out := plan_ptr
            }

            let __planOutOffset := skip_plan_names(__plan.offset)
            __planOut.offset := __planOutOffset
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
    }
}
