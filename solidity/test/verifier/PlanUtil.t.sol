// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {PlanUtil} from "../../src/verifier/PlanUtil.sol";
import {FF, F} from "../base/FieldUtil.sol";

contract PlanUtilTest is Test {
    struct ColumnMetadata {
        uint64 tableIndex;
        bytes name;
        uint32 columnVariant;
    }

    function generatePlanPrefix(
        bytes[] memory tableNames,
        ColumnMetadata[] memory columns,
        bytes[] memory outputColumnNames
    ) public pure returns (bytes memory result) {
        uint64 numberOfTables = uint64(tableNames.length);
        result = abi.encodePacked(numberOfTables);
        for (uint256 i = 0; i < numberOfTables; ++i) {
            result = bytes.concat(result, abi.encodePacked(uint64(tableNames[i].length), tableNames[i]));
        }
        uint64 numberOfColumns = uint64(columns.length);
        result = bytes.concat(result, abi.encodePacked(numberOfColumns));
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            result = bytes.concat(
                result,
                abi.encodePacked(
                    columns[i].tableIndex, uint64(columns[i].name.length), columns[i].name, columns[i].columnVariant
                )
            );
        }
        uint64 numberOfOutputColumns = uint64(outputColumnNames.length);
        result = bytes.concat(result, abi.encodePacked(numberOfOutputColumns));
        for (uint256 i = 0; i < numberOfOutputColumns; ++i) {
            result = bytes.concat(result, abi.encodePacked(uint64(outputColumnNames[i].length), outputColumnNames[i]));
        }
    }

    function testSkipSimplePlanPrefix() public pure {
        bytes[] memory tableNames = new bytes[](2);
        tableNames[0] = "A";
        tableNames[1] = "B2";
        ColumnMetadata[] memory columns = new ColumnMetadata[](2);
        columns[0] = ColumnMetadata(0, "A", 5);
        columns[1] = ColumnMetadata(1, "B2", 5);
        bytes[] memory outputColumnNames = new bytes[](2);
        outputColumnNames[0] = "A";
        outputColumnNames[1] = "B2";
        bytes memory planPrefix = generatePlanPrefix(tableNames, columns, outputColumnNames);
        bytes memory planPostfix = hex"abcdef";
        bytes memory plan = bytes.concat(planPrefix, planPostfix);
        bytes memory resultingPlan = PlanUtil.__skipPlanNames(plan);
        assertEq(resultingPlan.length, planPostfix.length);
        uint256 length = resultingPlan.length;
        for (uint256 i = 0; i < length; ++i) {
            assertEq(resultingPlan[i], planPostfix[i]);
        }
    }

    function testFuzzSkipPlanPrefix(
        bytes[] memory tableNames,
        ColumnMetadata[] memory columns,
        bytes[] memory outputColumnNames,
        bytes memory planPostfix
    ) public pure {
        bytes memory planPrefix = generatePlanPrefix(tableNames, columns, outputColumnNames);
        bytes memory plan = bytes.concat(planPrefix, planPostfix);
        bytes memory resultingPlan = PlanUtil.__skipPlanNames(plan);
        assertEq(resultingPlan.length, planPostfix.length);
        uint256 length = resultingPlan.length;
        for (uint256 i = 0; i < length; ++i) {
            assertEq(resultingPlan[i], planPostfix[i]);
        }
    }
}
