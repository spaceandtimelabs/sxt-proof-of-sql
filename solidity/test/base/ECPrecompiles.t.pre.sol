// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import { ECPrecompiles } from "../../src/base/ECPrecompiles.pre.sol";

library ECPrecompilesTestWrapper {
    function ecAdd(uint256[4] calldata args) external view {
        ECPrecompiles.__ecAdd(args);
    }

    function ecMul(uint256[3] calldata args) external view {
        ECPrecompiles.__ecMul(args);
    }

    function ecPairingX2(uint256[12] calldata args) external view returns (uint256 success) {
        success = ECPrecompiles.__ecPairingX2(args);
    }
}

library ECPrecompilesTestHelper {
    function ecBasePower(uint256 e) public view returns (uint256 x, uint256 y) {
        uint256[3] memory scratch = [G1_GEN_X, G1_GEN_Y, e % MODULUS];
        assembly {
            pop(staticcall(ECMUL_GAS, ECMUL_ADDRESS, scratch, WORDX3_SIZE, scratch, WORDX2_SIZE))
        }
        x = scratch[0];
        y = scratch[1];
    }
}

contract ECPrecompilesTest is Test {
    function testECAdd() public view {
        uint256[4] memory argsPtr = [uint256(1), 2, 1, 2];
        ECPrecompiles.__ecAdd(argsPtr);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);

        argsPtr = [uint256(0), 0, 1, 2];
        ECPrecompiles.__ecAdd(argsPtr);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
    }

    function testFuzzECAdd(uint256 a, uint256 b) public view {
        uint256 c = addmod(a, b, MODULUS);
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 bx, uint256 by) = ECPrecompilesTestHelper.ecBasePower(b);
        (uint256 cx, uint256 cy) = ECPrecompilesTestHelper.ecBasePower(c);
        uint256[4] memory argsPtr = [ax, ay, bx, by];
        ECPrecompiles.__ecAdd(argsPtr);
        assert(argsPtr[0] == cx);
        assert(argsPtr[1] == cy);
        // scratch space
        assert(argsPtr[2] == bx);
        assert(argsPtr[3] == by);
    }

    function testFuzzECAddInfinity(uint256 a) public view {
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        uint256[4] memory argsPtr = [ax, ay, 0, 0];
        ECPrecompiles.__ecAdd(argsPtr);
        assert(argsPtr[0] == ax);
        assert(argsPtr[1] == ay);
        // scratch space
        assert(argsPtr[2] == 0);
        assert(argsPtr[3] == 0);
    }

    function testAddGenToNegGen() public view {
        uint256[4] memory argsPtr = [G1_GEN_X, G1_GEN_Y, G1_NEG_GEN_X, G1_NEG_GEN_Y];
        ECPrecompiles.__ecAdd(argsPtr);
        assert(argsPtr[0] == 0);
        assert(argsPtr[1] == 0);
        // scratch space
        assert(argsPtr[2] == G1_NEG_GEN_X);
        assert(argsPtr[3] == G1_NEG_GEN_Y);
    }

    function testRevertWhenECAddInvalidInput() public {
        uint256[4] memory argsPtr = [uint256(1), 2, 3, 4];
        vm.expectRevert(Errors.InvalidECAddInputs.selector);
        ECPrecompilesTestWrapper.ecAdd(argsPtr);
    }

    function testECMul() public view {
        uint256[3] memory argsPtr = [uint256(1), 2, 2];
        ECPrecompiles.__ecMul(argsPtr);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 2);

        argsPtr = [uint256(1), 2, 1];
        ECPrecompiles.__ecMul(argsPtr);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
    }

    function testFuzzECMul(uint256 a, uint256 e) public view {
        uint256 c = mulmod(a, e, MODULUS);
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 cx, uint256 cy) = ECPrecompilesTestHelper.ecBasePower(c);
        uint256[3] memory argsPtr = [ax, ay, e];
        ECPrecompiles.__ecMul(argsPtr);
        assert(argsPtr[0] == cx);
        assert(argsPtr[1] == cy);
        // scratch space
        assert(argsPtr[2] == e);
    }

    function testRevertWhenECMulInvalidInput() public {
        uint256[3] memory argsPtr = [uint256(2), 3, 4];
        vm.expectRevert(Errors.InvalidECMulInputs.selector);
        ECPrecompilesTestWrapper.ecMul(argsPtr);
    }

    function testECMulAssign() public view {
        uint256[3] memory argsPtr = [uint256(1), 2, 0xDEAD];
        ECPrecompiles.__ecMulAssign(argsPtr, 2);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 2);

        argsPtr = [uint256(1), 2, 0xDEAD];
        ECPrecompiles.__ecMulAssign(argsPtr, 1);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
    }

    function testFuzzECMulAssign(uint256 a, uint256 e) public view {
        uint256 c = mulmod(a, e, MODULUS);
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 cx, uint256 cy) = ECPrecompilesTestHelper.ecBasePower(c);
        uint256[3] memory argsPtr = [ax, ay, 0xDEAD];
        ECPrecompiles.__ecMulAssign(argsPtr, e);
        assert(argsPtr[0] == cx);
        assert(argsPtr[1] == cy);
        // scratch space
        assert(argsPtr[2] == e);
    }

    function testECPairingX2() public view {
        uint256[12] memory argsPtr = [
            0x2cf44499d5d27bb186308b7af7af02ac5bc9eeb6a3d147c186b21fb1b76e18da,
            0x2c0f001f52110ccfe69108924926e45f0b0c868df0e7bde1fe16d3242dc715f6,
            0x1fb19bb476f6b9e44e2a32234da8212f61cd63919354bc06aef31e3cfaff3ebc,
            0x22606845ff186793914e03e21df544c34ffe2f2f3504de8a79d9159eca2d98d9,
            0x2bd368e28381e8eccb5fa81fc26cf3f048eea9abfdd85d7ed3ab3698d63e4f90,
            0x2fe02e47887507adf0ff1743cbac6ba291e66f59be6bd763950bb16041a0a85e,
            0x0000000000000000000000000000000000000000000000000000000000000001,
            0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd45,
            0x1971ff0471b09fa93caaf13cbf443c1aede09cc4328f5a62aad45f40ec133eb4,
            0x091058a3141822985733cbdddfed0fd8d6c104e9e9eff40bf5abfef9ab163bc7,
            0x2a23af9a5ce2ba2796c1f4e453a370eb0af8c212d9dc9acd8fc02c2e907baea2,
            0x23a8eb0b0996252cb548a4487da97b02422ebc0e834613f954de6c7e0afdc1fc
        ];
        assert(ECPrecompiles.__ecPairingX2(argsPtr) == 1);
    }

    function testG1GenIsNontrivial() public view {
        uint256[12] memory argsPtr = [
            G1_GEN_X,
            G1_GEN_Y,
            G2_GEN_X_IMAG,
            G2_GEN_X_REAL,
            G2_GEN_Y_IMAG,
            G2_GEN_Y_REAL,
            G1_GEN_X,
            G1_GEN_Y,
            G2_GEN_X_IMAG,
            G2_GEN_X_REAL,
            G2_GEN_Y_IMAG,
            G2_GEN_Y_REAL
        ];
        assert(ECPrecompiles.__ecPairingX2(argsPtr) == 0);
    }

    function testG1NegGenIsCorrect() public view {
        uint256[12] memory argsPtr = [
            G1_GEN_X,
            G1_GEN_Y,
            G2_GEN_X_IMAG,
            G2_GEN_X_REAL,
            G2_GEN_Y_IMAG,
            G2_GEN_Y_REAL,
            G1_GEN_X,
            G1_GEN_Y,
            G2_NEG_GEN_X_IMAG,
            G2_NEG_GEN_X_REAL,
            G2_NEG_GEN_Y_IMAG,
            G2_NEG_GEN_Y_REAL
        ];
        assert(ECPrecompiles.__ecPairingX2(argsPtr) == 1);
    }

    function testFuzzECPairingX2WithValidInputsThatDoNotSumToZero(uint256 a, uint256 b) public view {
        vm.assume(a != 0 || b != 0);
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 bx, uint256 by) = ECPrecompilesTestHelper.ecBasePower(b);
        uint256[12] memory argsPtr = [
            ax,
            ay,
            G2_GEN_X_IMAG,
            G2_GEN_X_REAL,
            G2_GEN_Y_IMAG,
            G2_GEN_Y_REAL,
            bx,
            by,
            G2_GEN_X_IMAG,
            G2_GEN_X_REAL,
            G2_GEN_Y_IMAG,
            G2_GEN_Y_REAL
        ];
        assert(ECPrecompiles.__ecPairingX2(argsPtr) == 0);
    }

    function testFuzzECPairingX2WithValidInputsThatDoSumToZero(uint256 a) public view {
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 bx, uint256 by) = ECPrecompilesTestHelper.ecBasePower(MODULUS - (a % MODULUS));
        uint256[12] memory argsPtr = [
            ax,
            ay,
            G2_GEN_X_IMAG,
            G2_GEN_X_REAL,
            G2_GEN_Y_IMAG,
            G2_GEN_Y_REAL,
            bx,
            by,
            G2_GEN_X_IMAG,
            G2_GEN_X_REAL,
            G2_GEN_Y_IMAG,
            G2_GEN_Y_REAL
        ];
        assert(ECPrecompiles.__ecPairingX2(argsPtr) == 1);
        uint256[12] memory argsPtr2 = [
            ax,
            ay,
            G2_GEN_X_IMAG,
            G2_GEN_X_REAL,
            G2_GEN_Y_IMAG,
            G2_GEN_Y_REAL,
            ax,
            ay,
            G2_NEG_GEN_X_IMAG,
            G2_NEG_GEN_X_REAL,
            G2_NEG_GEN_Y_IMAG,
            G2_NEG_GEN_Y_REAL
        ];
        assert(ECPrecompiles.__ecPairingX2(argsPtr2) == 1);
    }

    function testFuzzECPairingX2RevertsWithInvalidInput(uint256[12] memory argsPtr) public {
        for (uint256 i = 0; i < 12; ++i) {
            vm.assume(argsPtr[i] != 0);
        }
        vm.expectRevert(Errors.InvalidECPairingInputs.selector);
        ECPrecompilesTestWrapper.ecPairingX2(argsPtr);
    }

    function testRevertWhenECPairingX2WithSimpleInvalidInput() public {
        uint256[12] memory argsPtr = [uint256(1), 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        vm.expectRevert(Errors.InvalidECPairingInputs.selector);
        ECPrecompilesTestWrapper.ecPairingX2(argsPtr);
    }

    function testCalldataECAddAssign() public view {
        uint256[4] memory argsPtr = [uint256(1), 2, 0x5C, 0xDEAD];
        argsPtr = ECPrecompiles.__calldataECAddAssign(argsPtr, [uint256(1), 2]);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.__calldataECAddAssign(argsPtr, [uint256(1), 2]);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
    }

    function testFuzzCalldataECAddAssign(uint256 a, uint256 b) public view {
        uint256 c = addmod(a, b, MODULUS);
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 bx, uint256 by) = ECPrecompilesTestHelper.ecBasePower(b);
        (uint256 cx, uint256 cy) = ECPrecompilesTestHelper.ecBasePower(c);
        uint256[4] memory argsPtr = [ax, ay, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.__calldataECAddAssign(argsPtr, [bx, by]);
        assert(argsPtr[0] == cx);
        assert(argsPtr[1] == cy);
        // scratch space
        assert(argsPtr[2] == bx);
        assert(argsPtr[3] == by);
    }

    function testCalldataECMulAddAssign() public view {
        uint256[5] memory argsPtr = [uint256(1), 2, 0xDEAD, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.__calldataECMulAddAssign(argsPtr, [uint256(1), 2], 1);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
        assert(argsPtr[4] == 1);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.__calldataECMulAddAssign(argsPtr, [uint256(1), 2], 2);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[3] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        assert(argsPtr[4] == 2);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.__calldataECMulAddAssign(argsPtr, [uint256(1), 2], 1);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
        assert(argsPtr[4] == 1);
    }

    function testFuzzCalldataECMulAddAssign(uint256 a, uint256 b, uint256 e) public view {
        uint256 be = mulmod(b, e, MODULUS);
        uint256 c = addmod(a, be, MODULUS);
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 bx, uint256 by) = ECPrecompilesTestHelper.ecBasePower(b);
        (uint256 cx, uint256 cy) = ECPrecompilesTestHelper.ecBasePower(c);
        uint256[5] memory argsPtr = [ax, ay, 0xDEAD, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.__calldataECMulAddAssign(argsPtr, [bx, by], e);
        assert(argsPtr[0] == cx);
        assert(argsPtr[1] == cy);
        // scratch space shows intermediate result of b * e
        (uint256 bex, uint256 bey) = ECPrecompilesTestHelper.ecBasePower(be);
        assert(argsPtr[2] == bex);
        assert(argsPtr[3] == bey);
        assert(argsPtr[4] == e);
    }

    function testConstantECMulAddAssign() public view {
        uint256[5] memory argsPtr = [uint256(1), 2, 0xDEAD, 0xDEAD, 0xDEAD];
        ECPrecompiles.__constantECMulAddAssign(argsPtr, 1, 2, 1);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
        assert(argsPtr[4] == 1);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD, 0xDEAD];
        ECPrecompiles.__constantECMulAddAssign(argsPtr, 1, 2, 2);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[3] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        assert(argsPtr[4] == 2);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD, 0xDEAD];
        ECPrecompiles.__constantECMulAddAssign(argsPtr, 1, 2, 1);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
        assert(argsPtr[4] == 1);
    }

    function testFuzzConstantECMulAddAssign(uint256 a, uint256 b, uint256 e) public view {
        uint256 be = mulmod(b, e, MODULUS);
        uint256 c = addmod(a, be, MODULUS);
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 bx, uint256 by) = ECPrecompilesTestHelper.ecBasePower(b);
        (uint256 cx, uint256 cy) = ECPrecompilesTestHelper.ecBasePower(c);
        uint256[5] memory argsPtr = [ax, ay, 0xDEAD, 0xDEAD, 0xDEAD];
        ECPrecompiles.__constantECMulAddAssign(argsPtr, bx, by, e);
        assert(argsPtr[0] == cx);
        assert(argsPtr[1] == cy);
        // scratch space shows intermediate result of b * e
        (uint256 bex, uint256 bey) = ECPrecompilesTestHelper.ecBasePower(be);
        assert(argsPtr[2] == bex);
        assert(argsPtr[3] == bey);
        assert(argsPtr[4] == e);
    }

    function testECAddAssign() public view {
        uint256[4] memory argsPtr = [uint256(1), 2, 0xDEAD, 0xDEAD];
        uint256[2] memory point = [uint256(1), 2];
        ECPrecompiles.__ecAddAssign(argsPtr, point);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD];
        ECPrecompiles.__ecAddAssign(argsPtr, point);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
    }

    function testFuzzECAddAssign(uint256 a, uint256 b) public view {
        uint256 c = addmod(a, b, MODULUS);
        (uint256 ax, uint256 ay) = ECPrecompilesTestHelper.ecBasePower(a);
        (uint256 bx, uint256 by) = ECPrecompilesTestHelper.ecBasePower(b);
        (uint256 cx, uint256 cy) = ECPrecompilesTestHelper.ecBasePower(c);
        uint256[4] memory argsPtr = [ax, ay, 0xDEAD, 0xDEAD];
        uint256[2] memory point = [bx, by];
        ECPrecompiles.__ecAddAssign(argsPtr, point);
        assert(argsPtr[0] == cx);
        assert(argsPtr[1] == cy);
        // scratch space
        assert(argsPtr[2] == bx);
        assert(argsPtr[3] == by);
    }

    function testGeneratorsAreGenerators() public view {
        // Check that the generators are not the identity. i.e., that 2gen != gen.
        uint256[4] memory argsPtr = [G1_GEN_X, G1_GEN_Y, G1_GEN_X, G1_GEN_Y];
        ECPrecompiles.__ecAdd(argsPtr);
        assert(argsPtr[0] != G1_GEN_X);
        assert(argsPtr[1] != G1_GEN_Y);
        // scratch space
        assert(argsPtr[2] == G1_GEN_X);
        assert(argsPtr[3] == G1_GEN_Y);
        // Check that the generator has order that divides the modulus. i.e., that (modulus+1)*gen == gen.
        uint256[3] memory argsPtr2 = [G1_GEN_X, G1_GEN_Y, MODULUS + 1];
        ECPrecompiles.__ecMul(argsPtr2);
        assert(argsPtr2[0] == G1_GEN_X);
        assert(argsPtr2[1] == G1_GEN_Y);
        // scratch space
        assert(argsPtr2[2] == MODULUS + 1);
    }
}
