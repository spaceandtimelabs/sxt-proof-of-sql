// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {ColumnExpr} from "../../src/proof_exprs/ColumnExpr.pre.sol";
import {VerificationBuilder} from "../../src/proof/VerificationBuilder.pre.sol";

contract ColumnExprTest is Test {
    function testColumnExpr() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory values = new uint256[](3);
        values[0] = 0x11111111;
        values[1] = 0x22222222;
        values[2] = 0x33333333;
        builder.columnEvaluations = values;

        bytes memory exprIn = abi.encodePacked(uint64(1), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";

        (bytes memory exprOut, uint256 eval) = ColumnExpr.__columnExprEvaluate(exprIn, builder);
        assert(eval == 0x22222222);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testFuzzColumnExpr(uint64 columnNum, bytes memory trailingExpr, uint256[] memory columnValues)
        public
        pure
    {
        vm.assume(columnNum < columnValues.length);

        VerificationBuilder.Builder memory builder;
        builder.columnEvaluations = columnValues;

        bytes memory exprIn = abi.encodePacked(columnNum, trailingExpr);
        (bytes memory exprOut, uint256 eval) = ColumnExpr.__columnExprEvaluate(exprIn, builder);

        assert(eval == columnValues[columnNum]);
        assert(exprOut.length == trailingExpr.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == trailingExpr[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testInvalidColumnIndex() public {
        VerificationBuilder.Builder memory builder;
        uint256[] memory values = new uint256[](2);
        builder.columnEvaluations = values;

        bytes memory exprIn = abi.encodePacked(uint64(2), hex"abcdef");
        vm.expectRevert(Errors.InvalidIndex.selector);
        ColumnExpr.__columnExprEvaluate(exprIn, builder);
    }
}
