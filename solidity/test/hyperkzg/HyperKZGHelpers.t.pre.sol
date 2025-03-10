// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import "../../src/base/Errors.sol";
import {HyperKZGHelpers} from "../../src/hyperkzg/HyperKZGHelpers.pre.sol";
import {ECPrecompilesTestHelper} from "../base/ECPrecompiles.t.pre.sol";
import {F, FF} from "../base/FieldUtil.sol";

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
        FF expectedSum = F.ZERO;
        uint256 ell = v.length;
        for (uint256 i = 0; i < ell; ++i) {
            for (uint256 j = 0; j < 3; ++j) {
                FF qdPow = F.ONE;
                for (uint256 k = 0; k < i; ++k) {
                    qdPow = qdPow * F.from(q);
                }
                for (uint256 k = 0; k < j; ++k) {
                    qdPow = qdPow * F.from(d);
                }
                expectedSum = expectedSum + F.from(v[i][j]) * qdPow;
            }
        }
        assert(HyperKZGHelpers.__bivariateEvaluation(v, q, d) == expectedSum.into());
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
            FF vRand0 = F.from(vRand[i][0]);
            FF vRand1 = F.from(vRand[i][1]);
            FF xi = F.from(x[i]);
            v[i][0] = (F.TWO * F.from(r) * vRand0).into();
            v[i][1] = (F.TWO * F.from(r) * vRand1).into();
            y = (F.from(r) * (F.ONE - xi) * (vRand0 + vRand1) + xi * (vRand0 - vRand1)).into();
            if (i < x.length - 1) {
                v[i + 1][2] = y;
            }
        }
        HyperKZGHelpers.__checkVConsistency(v, r, x, y);
    }

    /// forge-config: default.fuzz.max-test-rejects = 100000
    function testFuzzRevertsVConsistency(uint256[3][] calldata v, uint256 r, uint256[] memory x, uint256 y) public {
        vm.assume(x.length > 0);
        uint256 xLength = x.length;
        vm.assume(v.length == xLength);
        bool xrAllZero = r == 0;
        for (uint256 i = 0; i < xLength; ++i) {
            xrAllZero = xrAllZero && (x[i] == 0);
        }
        vm.assume(!xrAllZero);
        vm.expectRevert(Errors.HyperKZGInconsistentV.selector);
        HyperKZGHelpers.__checkVConsistency(v, r, x, y);
    }

    function testEmptyUnivariateGroupEvaluation() public view {
        uint256[4] memory scratch = [uint256(0xDEAD), 0xDEAD, 0xDEAD, 0xDEAD];
        uint256[2][] memory g = new uint256[2][](0);
        scratch = HyperKZGHelpers.__univariateGroupEvaluation(g, 7, scratch);
        assert(scratch[0] == 0);
        assert(scratch[1] == 0);
        // scratch space
        assert(scratch[2] == 0xDEAD);
        assert(scratch[3] == 0xDEAD);
    }

    function testSmallUnivariateGroupEvaluation() public view {
        uint256[4] memory scratch = [uint256(0xDEAD), 0xDEAD, 0xDEAD, 0xDEAD];
        uint256[2][] memory g = new uint256[2][](1);
        (uint256 gx, uint256 gy) = ECPrecompilesTestHelper.ecBasePower(2);
        g[0] = [gx, gy];
        scratch = HyperKZGHelpers.__univariateGroupEvaluation(g, 7, scratch);
        (gx, gy) = ECPrecompilesTestHelper.ecBasePower(2);
        assert(scratch[0] == gx);
        assert(scratch[1] == gy);
        // scratch space
        assert(scratch[2] == 0xDEAD);
        assert(scratch[3] == 0xDEAD);
    }

    function testSimpleUnivariateGroupEvaluation() public view {
        uint256[4] memory scratch = [uint256(0xDEAD), 0xDEAD, 0xDEAD, 0xDEAD];
        uint256[2][] memory g = new uint256[2][](3);
        (uint256 gx, uint256 gy) = ECPrecompilesTestHelper.ecBasePower(2);
        g[0] = [gx, gy];
        (gx, gy) = ECPrecompilesTestHelper.ecBasePower(3);
        g[1] = [gx, gy];
        (gx, gy) = ECPrecompilesTestHelper.ecBasePower(5);
        g[2] = [gx, gy];
        scratch = HyperKZGHelpers.__univariateGroupEvaluation(g, 7, scratch);
        (gx, gy) = ECPrecompilesTestHelper.ecBasePower(2 + 3 * 7 + 5 * (7 ** 2));
        assert(scratch[0] == gx);
        assert(scratch[1] == gy);
        // scratch space
        assert(scratch[2] == g[0][0]);
        assert(scratch[3] == g[0][1]);
    }

    function testFuzzUnivariateGroupEvaluation(uint256[] memory p, uint256 e) public view {
        uint256[4] memory scratch = [uint256(0xDEAD), 0xDEAD, 0xDEAD, 0xDEAD];
        uint256[2][] memory g = new uint256[2][](p.length);
        FF pOfE = F.ZERO;
        uint256 n = p.length;
        FF eToTheI = F.ONE;
        uint256 gx;
        uint256 gy;
        for (uint256 i = 0; i < n; ++i) {
            pOfE = pOfE + F.from(p[i]) * eToTheI;
            (gx, gy) = ECPrecompilesTestHelper.ecBasePower(p[i]);
            g[i] = [gx, gy];
            eToTheI = eToTheI * F.from(e);
        }
        scratch = HyperKZGHelpers.__univariateGroupEvaluation(g, e, scratch);
        (gx, gy) = ECPrecompilesTestHelper.ecBasePower(pOfE.into());
        assert(scratch[0] == gx);
        assert(scratch[1] == gy);
        if (n > 1) {
            // scratch space
            assert(scratch[2] == g[0][0]);
            assert(scratch[3] == g[0][1]);
        } else {
            assert(scratch[2] == 0xDEAD);
            assert(scratch[3] == 0xDEAD);
        }
    }

    function testComputeGLMSMWithAllZeros() public view {
        uint256[2][] memory com = new uint256[2][](1);
        com[0] = [uint256(0), 0];
        uint256[2][3] memory w;
        w[0] = [uint256(0), 0];
        w[1] = [uint256(0), 0];
        w[2] = [uint256(0), 0];
        uint256[2] memory commitment = [uint256(0), 0];
        uint256[4] memory rqdb = [uint256(0), 0, 0, 0];
        uint256[5] memory scratch;
        scratch = HyperKZGHelpers.__computeGLMSM({
            __com: com,
            __w: w,
            __commitment: commitment,
            __rqdb: rqdb,
            __scratch: scratch
        });
        assert(scratch[0] == 0);
        assert(scratch[1] == 0);
    }

    function testComputeGLMSMWithSimpleValues() public view {
        uint256[2][] memory com = new uint256[2][](2);
        (uint256 comx, uint256 comy) = ECPrecompilesTestHelper.ecBasePower(2);
        com[0] = [comx, comy];
        (comx, comy) = ECPrecompilesTestHelper.ecBasePower(3);
        com[1] = [comx, comy];

        uint256[2][3] memory w;
        (uint256 wx, uint256 wy) = ECPrecompilesTestHelper.ecBasePower(5);
        w[0] = [wx, wy];
        (wx, wy) = ECPrecompilesTestHelper.ecBasePower(7);
        w[1] = [wx, wy];
        (wx, wy) = ECPrecompilesTestHelper.ecBasePower(11);
        w[2] = [wx, wy];

        (uint256 commitmentx, uint256 commitmenty) = ECPrecompilesTestHelper.ecBasePower(13);
        uint256[2] memory commitment = [commitmentx, commitmenty];

        uint256[4] memory rqdb = [uint256(17), 19, 23, 29];

        uint256[5] memory scratch;
        scratch = HyperKZGHelpers.__computeGLMSM({
            __com: com,
            __w: w,
            __commitment: commitment,
            __rqdb: rqdb,
            __scratch: scratch
        });

        uint256 expectedPower =
            (23 ** 2 + 23 + 1) * (13 + 2 * 19 + 3 * 19 ** 2) + (17 * 23) ** 2 * 11 - 17 * 23 * 7 + 17 * 5 - 29;
        (uint256 expectedx, uint256 expectedy) = ECPrecompilesTestHelper.ecBasePower(expectedPower);

        assert(scratch[0] == expectedx);
        assert(scratch[1] == expectedy);
    }

    function testFuzzComputeGLMSM(
        uint256[] memory comPower,
        uint256[3] memory wPower,
        uint256 commitmentPower,
        uint256[4] memory rqdb,
        uint256[5] memory scratch
    ) public view {
        FF expectedPower;
        {
            FF r = F.from(rqdb[0]);
            FF d = F.from(rqdb[2]);
            FF b = F.from(rqdb[3]);
            expectedPower = -b;
            expectedPower = expectedPower + r * F.from(wPower[0]);
            expectedPower = expectedPower - r * d * F.from(wPower[1]);
            expectedPower = expectedPower + r * d * r * d * F.from(wPower[2]);
        }

        FF qToTheIPlusOne = F.ONE;
        FF comSum = F.ZERO;

        uint256[2][] memory com = new uint256[2][](comPower.length);
        uint256 comLength = comPower.length;
        for (uint256 i = 0; i < comLength; ++i) {
            {
                (uint256 comx, uint256 comy) = ECPrecompilesTestHelper.ecBasePower(comPower[i]);
                com[i] = [comx, comy];
            }
            FF q = F.from(rqdb[1]);
            qToTheIPlusOne = qToTheIPlusOne * q;
            comSum = comSum + F.from(comPower[i]) * qToTheIPlusOne;
        }
        {
            FF d = F.from(rqdb[2]);
            expectedPower = expectedPower + (d * d + d + F.ONE) * (F.from(commitmentPower) + comSum);
        }

        uint256[2][3] memory w;
        for (uint256 i = 0; i < 3; ++i) {
            (uint256 wx, uint256 wy) = ECPrecompilesTestHelper.ecBasePower(wPower[i]);
            w[i] = [wx, wy];
        }

        (uint256 commitmentx, uint256 commitmenty) = ECPrecompilesTestHelper.ecBasePower(commitmentPower);
        uint256[2] memory commitment = [commitmentx, commitmenty];
        scratch = HyperKZGHelpers.__computeGLMSM({
            __com: com,
            __w: w,
            __commitment: commitment,
            __rqdb: rqdb,
            __scratch: scratch
        });

        (uint256 expectedx, uint256 expectedy) = ECPrecompilesTestHelper.ecBasePower(expectedPower.into());
        assert(scratch[0] == expectedx);
        assert(scratch[1] == expectedy);
    }
}
