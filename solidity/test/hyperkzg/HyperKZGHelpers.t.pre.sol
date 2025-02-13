// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import "../../src/base/Errors.sol";
import {HyperKZGHelpers} from "../../src/hyperkzg/HyperKZGHelpers.pre.sol";

contract HyperKZGHelpersTest is Test {
    function testFuzzRunTranscriptWhenEllIs1(uint256[3] memory v, uint256[6] memory w, uint256[1] memory transcript)
        public
        pure
    {
        bytes32 expectedR = keccak256(abi.encodePacked(transcript, hex""));
        bytes32 expectedQ = keccak256(abi.encodePacked(keccak256(abi.encodePacked(expectedR)), v));
        bytes32 expectedD = keccak256(abi.encodePacked(keccak256(abi.encodePacked(expectedQ)), w));
        (uint256 r, uint256 q, uint256 d) = HyperKZGHelpers.__runTranscript({
            __com: hex"",
            __v: abi.encodePacked(v),
            __w: abi.encodePacked(w),
            __transcript: transcript,
            __ell: 1
        });
        assert(r == uint256(expectedR) & MODULUS_MASK);
        assert(q == uint256(expectedQ) & MODULUS_MASK);
        assert(d == uint256(expectedD) & MODULUS_MASK);
    }

    function testFuzzRunTranscriptWhenEllIs2(
        uint256[2] memory com,
        uint256[6] memory v,
        uint256[6] memory w,
        uint256[1] memory transcript
    ) public pure {
        bytes32 expectedR = keccak256(abi.encodePacked(transcript, com));
        bytes32 expectedQ = keccak256(abi.encodePacked(keccak256(abi.encodePacked(expectedR)), v));
        bytes32 expectedD = keccak256(abi.encodePacked(keccak256(abi.encodePacked(expectedQ)), w));
        (uint256 r, uint256 q, uint256 d) = HyperKZGHelpers.__runTranscript({
            __com: abi.encodePacked(com),
            __v: abi.encodePacked(v),
            __w: abi.encodePacked(w),
            __transcript: transcript,
            __ell: 2
        });
        assert(r == uint256(expectedR) & MODULUS_MASK);
        assert(q == uint256(expectedQ) & MODULUS_MASK);
        assert(d == uint256(expectedD) & MODULUS_MASK);
    }

    function testFuzzRunTranscriptWhenEllIs3(
        uint256[4] memory com,
        uint256[9] memory v,
        uint256[6] memory w,
        uint256[1] memory transcript
    ) public pure {
        bytes32 expectedR = keccak256(abi.encodePacked(transcript, com));
        bytes32 expectedQ = keccak256(abi.encodePacked(keccak256(abi.encodePacked(expectedR)), v));
        bytes32 expectedD = keccak256(abi.encodePacked(keccak256(abi.encodePacked(expectedQ)), w));
        (uint256 r, uint256 q, uint256 d) = HyperKZGHelpers.__runTranscript({
            __com: abi.encodePacked(com),
            __v: abi.encodePacked(v),
            __w: abi.encodePacked(w),
            __transcript: transcript,
            __ell: 3
        });
        assert(r == uint256(expectedR) & MODULUS_MASK);
        assert(q == uint256(expectedQ) & MODULUS_MASK);
        assert(d == uint256(expectedD) & MODULUS_MASK);
    }

    function testFuzzRunTranscriptRandom(uint256[] calldata proof, uint256[1] memory transcript) public pure {
        vm.assume(proof.length > 8);
        uint256 ell = (proof.length - 4) / (5);
        bytes memory com = abi.encodePacked(proof[0:2 * (ell - 1)]);
        bytes memory v = abi.encodePacked(proof[2 * ell - 2:5 * ell - 2]);
        bytes memory w = abi.encodePacked(proof[5 * ell - 2:5 * ell + 4]);
        bytes32 expectedR = keccak256(abi.encodePacked(transcript, com));
        bytes32 expectedQ = keccak256(abi.encodePacked(keccak256(abi.encodePacked(expectedR)), v));
        bytes32 expectedD = keccak256(abi.encodePacked(keccak256(abi.encodePacked(expectedQ)), w));

        (uint256 r, uint256 q, uint256 d) =
            HyperKZGHelpers.__runTranscript({__com: com, __v: v, __w: w, __transcript: transcript, __ell: ell});
        assert(r == uint256(expectedR) & MODULUS_MASK);
        assert(q == uint256(expectedQ) & MODULUS_MASK);
        assert(d == uint256(expectedD) & MODULUS_MASK);
    }

    function testSmallBivariateEvaluation() public pure {
        uint256[3][] memory v = new uint256[3][](2);
        v[0] = [uint256(101), 102, 103];
        v[1] = [uint256(104), 105, 106];

        assert(
            HyperKZGHelpers.__bivariateEvaluation(v, 5, 7)
                == 101 * 1 + 102 * 7 + 103 * 49 + 104 * 5 + 105 * 35 + 106 * 245
        );
    }

    function testEmpty() public pure {
        uint256[3][] memory v = new uint256[3][](0);

        assert(HyperKZGHelpers.__bivariateEvaluation(v, 5, 7) == 0);
    }

    function testFuzzBivariateEvaluation(uint256[3][] calldata v, uint256 q, uint256 d) public pure {
        uint256 expectedSum = 0;
        uint256 ell = v.length;
        for (uint256 i = 0; i < ell; ++i) {
            for (uint256 j = 0; j < 3; ++j) {
                uint256 qdPow = 1;
                for (uint256 k = 0; k < i; ++k) {
                    qdPow = mulmod(qdPow, q, MODULUS);
                }
                for (uint256 k = 0; k < j; ++k) {
                    qdPow = mulmod(qdPow, d, MODULUS);
                }
                expectedSum = addmod(expectedSum, mulmod(v[i][j], qdPow, MODULUS), MODULUS);
            }
        }
        assert(HyperKZGHelpers.__bivariateEvaluation(v, q, d) == expectedSum);
    }

    function testEmptyCheckVConsistency() public pure {
        uint256[3][] memory v = new uint256[3][](0);
        uint256 r = 5;
        uint256[] memory x = new uint256[](0);
        uint256 y = 1234567890;
        HyperKZGHelpers.__checkVConsistency(v, r, x, y);
    }

    function testSimpleCheckVConsistency() public pure {
        uint256[3][] memory v = new uint256[3][](3);
        v[0] = [uint256(0), 0, 1234567890];
        v[1] = [uint256(1020), 1010, 0];
        v[2] = [uint256(1020), 1010, 5 * (102 + 101)];
        uint256 r = 5;
        uint256[] memory x = new uint256[](3);
        x[0] = 99999;
        x[1] = 0;
        x[2] = 1;
        uint256 y = 102 - 101;
        HyperKZGHelpers.__checkVConsistency(v, r, x, y);
    }

    function testFuzzCheckVConsistency(uint256[2][] memory vRand, uint256 r, uint256[] memory x, uint256 y)
        public
        pure
    {
        uint256 ell = x.length;
        vm.assume(ell > 0);
        vm.assume(vRand.length > ell);
        uint256[3][] memory v = new uint256[3][](ell);
        v[0][2] = vRand[ell][0];
        for (uint256 i = 0; i < ell; ++i) {
            // v_0 = 2r * vRand_0
            // v_1 = 2r * vRand_1
            v[i][0] = mulmod(mulmod(2, r, MODULUS), vRand[i][0], MODULUS);
            v[i][1] = mulmod(mulmod(2, r, MODULUS), vRand[i][1], MODULUS);
            // y =  r * (1 - x) * (vRand_0 + vRand_1) + x * (vRand_0 - vRand_1)
            y = addmod(
                mulmod(
                    r,
                    mulmod(
                        1 + mulmod(MODULUS_MINUS_ONE, x[i], MODULUS), addmod(vRand[i][0], vRand[i][1], MODULUS), MODULUS
                    ),
                    MODULUS
                ),
                mulmod(x[i], addmod(vRand[i][0], mulmod(MODULUS_MINUS_ONE, vRand[i][1], MODULUS), MODULUS), MODULUS),
                MODULUS
            );
            if (i < x.length - 1) {
                v[i + 1][2] = y;
            }
        }
        HyperKZGHelpers.__checkVConsistency(v, r, x, y);
    }

    /// forge-config: default.fuzz.max-test-rejects = 100000
    function testFuzzRevertsVConsistency(uint256[3][] calldata v, uint256 r, uint256[] memory x, uint256 y) public {
        vm.assume(x.length > 0);
        vm.assume(v.length == x.length);
        vm.expectRevert(Errors.HyperKZGInconsistentV.selector);
        HyperKZGHelpers.__checkVConsistency(v, r, x, y);
    }
}
