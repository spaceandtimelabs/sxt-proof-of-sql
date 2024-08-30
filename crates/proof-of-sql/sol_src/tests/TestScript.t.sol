// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

library TestScript {
    struct CustomStruct {
        uint256 value;
    }

    error DummyError();

    function rustTestWeCanThrowErrorDependingOnParameter(uint256 x) public pure {
        if (x != 1234) revert DummyError();
    }

    function rustTestWeCanAcceptCustomStructAsEncodedBytes(bytes memory x) public pure {
        (CustomStruct memory y) = abi.decode(x, (CustomStruct)); // solhint-disable-line no-unused-vars
        if (y.value != 1234) revert DummyError();
    }

    function testWeCanRunForgeTests() public pure {} // solhint-disable-line no-empty-blocks
}
