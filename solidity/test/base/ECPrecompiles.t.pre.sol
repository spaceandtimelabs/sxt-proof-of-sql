// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import {ECPrecompiles} from "../../src/base/ECPrecompiles.pre.sol";

contract ECPrecompilesTest {
    function testECAdd() public view {
        uint256[4] memory argsPtr = [uint256(1), 2, 1, 2];
        ECPrecompiles.ecAdd(argsPtr);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);

        argsPtr = [uint256(0), 0, 1, 2];
        ECPrecompiles.ecAdd(argsPtr);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
    }

    function testECMul() public view {
        uint256[3] memory argsPtr = [uint256(1), 2, 2];
        ECPrecompiles.ecMul(argsPtr);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 2);

        argsPtr = [uint256(1), 2, 1];
        ECPrecompiles.ecMul(argsPtr);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
    }

    function testECMulAssign() public view {
        uint256[3] memory argsPtr = [uint256(1), 2, 0xDEAD];
        ECPrecompiles.ecMulAssign(argsPtr, 2);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 2);

        argsPtr = [uint256(1), 2, 0xDEAD];
        ECPrecompiles.ecMulAssign(argsPtr, 1);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
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
        assert(ECPrecompiles.ecPairingX2(argsPtr) == 1);
    }

    function testCalldataECAddAssign() public view {
        uint256[4] memory argsPtr = [uint256(1), 2, 0x5C, 0xDEAD];
        argsPtr = ECPrecompiles.calldataECAddAssign(argsPtr, [uint256(1), 2]);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.calldataECAddAssign(argsPtr, [uint256(1), 2]);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
    }

    function testCalldataECMulAddAssign() public view {
        uint256[5] memory argsPtr = [uint256(1), 2, 0xDEAD, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.calldataECMulAddAssign(argsPtr, [uint256(1), 2], 1);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
        assert(argsPtr[4] == 1);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.calldataECMulAddAssign(argsPtr, [uint256(1), 2], 2);
        assert(argsPtr[0] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[1] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        // scratch space
        assert(argsPtr[2] == 0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3);
        assert(argsPtr[3] == 0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4);
        assert(argsPtr[4] == 2);

        argsPtr = [uint256(0), 0, 0xDEAD, 0xDEAD, 0xDEAD];
        argsPtr = ECPrecompiles.calldataECMulAddAssign(argsPtr, [uint256(1), 2], 1);
        assert(argsPtr[0] == 1);
        assert(argsPtr[1] == 2);
        // scratch space
        assert(argsPtr[2] == 1);
        assert(argsPtr[3] == 2);
        assert(argsPtr[4] == 1);
    }
}
