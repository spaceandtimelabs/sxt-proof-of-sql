// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {Errors} from "../../src/base/Errors.sol";
import "../../src/base/Queue.pre.sol";

contract ErrorsTest is Test {
    /// forge-config: default.allow_internal_expect_revert = true
    function testEmptyDequeue() public {
        uint256[][1] memory queue = [new uint256[](0)];
        vm.expectRevert(Errors.EmptyQueue.selector);
        Queue.__dequeue(queue);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testDequeue() public {
        uint256[][1] memory queue = [new uint256[](3)];
        queue[0][0] = 1001;
        queue[0][1] = 1002;
        queue[0][2] = 1003;
        assert(Queue.__dequeue(queue) == 1001);
        assert(Queue.__dequeue(queue) == 1002);
        assert(Queue.__dequeue(queue) == 1003);
        vm.expectRevert(Errors.EmptyQueue.selector);
        Queue.__dequeue(queue);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzDequeue(uint256[][1] memory queue) public {
        uint256 length = queue[0].length;
        uint256[] memory original = new uint256[](length);
        for (uint256 i = 0; i < length; ++i) {
            original[i] = queue[0][i];
        }
        for (uint256 i = 0; i < length; ++i) {
            assert(Queue.__dequeue(queue) == original[i]);
        }
        vm.expectRevert(Errors.EmptyQueue.selector);
        Queue.__dequeue(queue);
    }
}
