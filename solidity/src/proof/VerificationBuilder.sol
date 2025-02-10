// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

// solhint-disable-next-line no-unused-import
import {FREE_PTR, VERIFICATION_BUILDER_SIZE} from "../base/Constants.sol";

library VerificationBuilder {
    /// @notice Allocates and reserves a block of memory for a verification builder.
    /// @return __builderPtr The pointer to the allocated builder region.
    function __allocate() internal pure returns (uint256 __builderPtr) {
        assembly {
            function builder_allocate() -> builder_ptr {
                builder_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(builder_ptr, VERIFICATION_BUILDER_SIZE))
            }
            __builderPtr := builder_allocate()
        }
    }
}
