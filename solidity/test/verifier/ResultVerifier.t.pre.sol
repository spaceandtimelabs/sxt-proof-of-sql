// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import "../../src/base/LagrangeBasisEvaluation.pre.sol";
import {ResultVerifier} from "../../src/verifier/ResultVerifier.pre.sol";
import {FF, F} from "../base/FieldUtil.sol";

contract ResultVerifierTest is Test {
    /// Utility for generating a serailized result
    function generateResult(
        int64[][] memory table,
        bytes[] memory columnNames,
        uint8 quoteType,
        uint32[] memory columnVariant
    ) public pure returns (bytes memory result) {
        uint64 numberOfColumns = uint64(table.length);
        result = abi.encodePacked(numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            uint64 numberOfRows = uint64(table[i].length);
            result = bytes.concat(
                result,
                abi.encodePacked(
                    uint64(columnNames[i].length), columnNames[i], quoteType, columnVariant[i], numberOfRows
                )
            );
            for (uint256 j = 0; j < numberOfRows; ++j) {
                int64 raw = table[i][j];
                if (columnVariant[i] == COLUMN_BIGINT_VARIANT) {
                    result = bytes.concat(result, bytes8(uint64(raw)));
                } else if (columnVariant[i] == COLUMN_INT_VARIANT) {
                    result = bytes.concat(result, bytes4(uint32(uint64(raw))));
                } else if (columnVariant[i] == COLUMN_SMALLINT_VARIANT) {
                    result = bytes.concat(result, bytes2(uint16(uint64(raw))));
                } else if (columnVariant[i] == COLUMN_TINYINT_VARIANT) {
                    result = bytes.concat(result, bytes1(uint8(uint64(raw))));
                } else {
                    revert("unsupported variant");
                }
            }
        }
    }

    /// Utility for evaluating a table at a given point. This is incredibly inefficient.
    function evaluateTable(int64[][] memory table, uint256[] memory evaluationPoint)
        public
        pure
        returns (uint256[] memory evaluations)
    {
        uint64 numberOfColumns = uint64(table.length);
        evaluations = new uint256[](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            uint64 numberOfRows = uint64(table[i].length);
            uint256[] memory evaluationVector =
                LagrangeBasisEvaluation.__computeEvaluationVec(numberOfRows, evaluationPoint);
            FF evaluation = F.ZERO;
            for (uint256 j = 0; j < numberOfRows; ++j) {
                evaluation = evaluation + F.from(table[i][j]) * F.from(evaluationVector[j]);
            }
            evaluations[i] = evaluation.into();
        }
    }

    function testCorrectResult() public pure {
        uint64 numberOfColumns = 4;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        uint32[] memory columnVariants = new uint32[](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "INT64";
        table[0][0] = 9223372036854775807;
        table[0][1] = -9223372036854775808;
        table[0][2] = -64;

        columnNames[1] = "INT32";
        table[1][0] = -2147483648;
        table[1][1] = 2147483647;
        table[1][2] = 32;

        columnNames[2] = "INT16";
        table[2][0] = 32767;
        table[2][1] = -32768;
        table[2][2] = -16;

        columnNames[3] = "INT8";
        table[3][0] = -128;
        table[3][1] = 127;
        table[3][2] = 8;

        columnVariants[0] = COLUMN_BIGINT_VARIANT;
        columnVariants[1] = COLUMN_INT_VARIANT;
        columnVariants[2] = COLUMN_SMALLINT_VARIANT;
        columnVariants[3] = COLUMN_TINYINT_VARIANT;

        bytes memory result = generateResult(table, columnNames, 0, columnVariants);

        uint256[] memory evaluationPoint = new uint256[](4);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;
        evaluationPoint[2] = 103;
        evaluationPoint[3] = 104;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testIncorrectResult() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        uint32[] memory columnVariants = new uint32[](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        columnVariants[0] = COLUMN_BIGINT_VARIANT;
        columnVariants[1] = COLUMN_SMALLINT_VARIANT;

        bytes memory result = generateResult(table, columnNames, 0, columnVariants);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);
        ++evaluations[0];

        vm.expectRevert(Errors.IncorrectResult.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testResultColumnMismatch() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        uint32[] memory columnVariants = new uint32[](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        columnVariants[0] = COLUMN_BIGINT_VARIANT;
        columnVariants[1] = COLUMN_SMALLINT_VARIANT;

        bytes memory result = generateResult(table, columnNames, 0, columnVariants);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);
        uint256[] memory wrongEvaluations = new uint256[](1);
        wrongEvaluations[0] = evaluations[0];

        vm.expectRevert(Errors.ResultColumnCountMismatch.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, wrongEvaluations);
    }

    function testInvalidColumnNameInResult() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        uint32[] memory columnVariants = new uint32[](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        columnVariants[0] = COLUMN_BIGINT_VARIANT;
        columnVariants[1] = COLUMN_SMALLINT_VARIANT;

        bytes memory result = generateResult(table, columnNames, 1, columnVariants);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        vm.expectRevert(Errors.InvalidResultColumnName.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testInvalidColumnTypeInResult() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        uint32[] memory columnVariants = new uint32[](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        columnVariants[0] = COLUMN_BIGINT_VARIANT;
        columnVariants[1] = 100;

        bytes memory result = generateResult(table, columnNames, 0, columnVariants);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        vm.expectRevert(Errors.UnsupportedColumnVariant.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testInconsistentResultColumnLengths() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        uint32[] memory columnVariants = new uint32[](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1] = new int64[](2);
        table[1][0] = -4;
        table[1][1] = 5;

        columnVariants[0] = COLUMN_BIGINT_VARIANT;
        columnVariants[1] = COLUMN_SMALLINT_VARIANT;

        bytes memory result = generateResult(table, columnNames, 0, columnVariants);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        vm.expectRevert(Errors.InconsistentResultColumnLengths.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testFuzzCorrectResult(
        uint64 numberOfColumns,
        uint64 numberOfRows,
        bytes[] memory nameData,
        int64[] memory data,
        uint32[] memory columnVariants,
        uint256[] memory evaluationPoint
    ) public pure {
        // solhint-disable-next-line gas-strict-inequalities
        vm.assume(nameData.length >= numberOfColumns);
        // solhint-disable-next-line gas-strict-inequalities
        vm.assume(data.length >= uint256(numberOfColumns) * uint256(numberOfRows));
        // solhint-disable-next-line gas-strict-inequalities
        vm.assume(columnVariants.length >= numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            vm.assume(columnVariants[i] < 4);
        }

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            columnNames[i] = nameData[i];
            table[i] = new int64[](numberOfRows);
            for (uint256 j = 0; j < numberOfRows; ++j) {
                int64 raw = data[i * uint256(numberOfRows) + j];
                if (columnVariants[i] == COLUMN_BIGINT_VARIANT) {
                    table[i][j] = int64(raw);
                } else if (columnVariants[i] == COLUMN_INT_VARIANT) {
                    // keep only the low 32 bits, then sign‐extend back to 64
                    table[i][j] = int64(int32(raw));
                } else if (columnVariants[i] == COLUMN_SMALLINT_VARIANT) {
                    table[i][j] = int64(int16(raw));
                } else if (columnVariants[i] == COLUMN_TINYINT_VARIANT) {
                    table[i][j] = int64(int8(raw));
                }
            }
        }

        bytes memory result = generateResult(table, columnNames, 0, columnVariants);

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }
}
