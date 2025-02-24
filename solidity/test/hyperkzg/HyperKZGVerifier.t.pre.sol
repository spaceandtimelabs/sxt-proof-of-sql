// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import "../../src/base/Errors.sol";
import {HyperKZGVerifier} from "../../src/hyperkzg/HyperKZGVerifier.pre.sol";

contract HyperKZGVerifierTest is Test {
    function _smallValidProof()
        internal
        pure
        returns (
            bytes memory proof,
            uint256[1] memory transcript,
            uint256[2] memory commitment,
            uint256[] memory x,
            uint256 y
        )
    {
        proof = hex"0b6f635ac169750af717f5d53eb0cf78f4d950ba03a8682dc64c999865e4fa35"
            hex"2c10edb249e2fb60808d4f3b341046234b642722fb52d8e5e9d93ea56d2cd568"
            hex"1aa53426df3662ebfa29e721651ace4ab0930f126e97d9244e119fe364f0ff17"
            hex"2866f444c0e85d2b451e62516979b5927a1f9780fe09e97cd9e06ce477247990"
            hex"0e0c52f1bad3c3a7130d06c3326082582bd72ee8ddca8d79de7e3e71927c3b8b"
            hex"038c3b66e85d4c2855d79f8d9412985a2fb626aba5889185e2bef8aa20065255"
            hex"2cd8130bf8d454016278a628ed6ec002f87dc19cd430df0b6122fce9cff9adba"
            hex"11c783363cfcb7c19385e58c6caa75b17183380465ae005b1f33396adb304f7c"
            hex"19679242d534d76ca5d3a612a9101559e599202b41c8ffea9d993abe744045be"
            hex"1d510b121ecf77874c871537b9f57ae762bad86775fb45bd16a0e244e3937460"
            hex"0ee3b4f118a676a84d7ef4da43fbf557b8edc75108fc8fcfd1d94b7181bd2bf6"
            hex"0ba843c2a13560d3775eb1bf14dbc87a1568ede645ba1f4095b0a2fe88ba527f"
            hex"304b78379710a379609aeaa7b1f4c0b50b725e7c70976aee099b6b54b74f2b53"
            hex"17a56970a14eb4457d2e0bf3d0109ffeb1001179d72bd6babdc5ed7bf98c5848";
        transcript = [0x9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658];
        commitment = [
            0x021c0c77e03d902e65cae960e0a053fa954bfa0d7f9241431bec21447ef8401d,
            0x09cf5acdf3c124d40247b54aebc809d93d40b7e0b0bf5a1499057bd499da528d
        ];
        x = new uint256[](2);
        x[0] = 0x7;
        x[1] = 0x5;
        y = 17;
    }

    function testVerifyHyperKZG() public view {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();
        HyperKZGVerifier.__verifyHyperKZG({
            __proof: proof,
            __transcript: transcript,
            __commitment: commitment,
            __x: x,
            __y: y
        });
    }

    function testVerifyHyperKZGRevertsIfInconsistentV() public {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();

        uint256 ell = x.length;
        uint256 vOffset = WORDX2_SIZE * ell - WORDX2_SIZE;

        // Tweak byte 4 of element 3 of v.
        proof[vOffset + 3 * WORD_SIZE + 4] ^= 0x10;

        vm.expectRevert(Errors.HyperKZGInconsistentV.selector);
        HyperKZGVerifier.__verifyHyperKZG({
            __proof: proof,
            __transcript: transcript,
            __commitment: commitment,
            __x: x,
            __y: y
        });
    }

    function testVerifyHyperKZGRevertsIfPairingCheckFailed() public {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();

        // Pick a group element that is wrong.
        commitment[0] = 1;
        commitment[1] = 2;

        vm.expectRevert(Errors.HyperKZGPairingCheckFailed.selector);
        HyperKZGVerifier.__verifyHyperKZG({
            __proof: proof,
            __transcript: transcript,
            __commitment: commitment,
            __x: x,
            __y: y
        });
    }

    function testVerifyHyperKZGRevertsIfEmptyPoint() public {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();

        // Empty x.
        x = new uint256[](0);
        vm.expectRevert(Errors.HyperKZGEmptyPoint.selector);
        HyperKZGVerifier.__verifyHyperKZG({
            __proof: proof,
            __transcript: transcript,
            __commitment: commitment,
            __x: x,
            __y: y
        });
    }

    function testVerifyHyperKZGRevertsIfProofIsTweaked() public {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();

        uint256 proofLength = proof.length;
        for (uint256 i = 0; i < proofLength; ++i) {
            for (uint8 j = 0; j < 8; ++j) {
                // Tweak
                proof[i] ^= bytes1(uint8(0x01) << j);
                vm.expectRevert();
                HyperKZGVerifier.__verifyHyperKZG({
                    __proof: proof,
                    __transcript: transcript,
                    __commitment: commitment,
                    __x: x,
                    __y: y
                });
                // Untweak
                proof[i] ^= bytes1(uint8(0x01) << j);
            }
        }
    }

    function testVerifyHyperKZGRevertsIfTranscriptIsTweaked() public {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();

        for (uint256 i = 0; i < 256; ++i) {
            // Tweak
            transcript[0] ^= 1 << i;
            vm.expectRevert();
            HyperKZGVerifier.__verifyHyperKZG({
                __proof: proof,
                __transcript: transcript,
                __commitment: commitment,
                __x: x,
                __y: y
            });
            // Untweak
            transcript[0] ^= 1 << i;
        }
    }

    function testVerifyHyperKZGRevertsIfCommitmentIsTweaked() public {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();

        for (uint256 i = 0; i < 2; ++i) {
            for (uint256 j = 0; j < 256; ++j) {
                // Tweak
                commitment[i] ^= 1 << j;
                vm.expectRevert();
                HyperKZGVerifier.__verifyHyperKZG({
                    __proof: proof,
                    __transcript: transcript,
                    __commitment: commitment,
                    __x: x,
                    __y: y
                });
                // Untweak
                commitment[i] ^= 1 << j;
            }
        }
    }

    function testVerifyHyperKZGRevertsIfXIsTweaked() public {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();

        uint256 ell = x.length;
        for (uint256 i = 0; i < ell; ++i) {
            for (uint256 j = 0; j < 256; ++j) {
                // Tweak
                x[i] ^= 1 << j;
                vm.expectRevert();
                HyperKZGVerifier.__verifyHyperKZG({
                    __proof: proof,
                    __transcript: transcript,
                    __commitment: commitment,
                    __x: x,
                    __y: y
                });
                // Untweak
                x[i] ^= 1 << j;
            }
        }
    }

    function testVerifyHyperKZGRevertsIfYIsTweaked() public {
        (bytes memory proof, uint256[1] memory transcript, uint256[2] memory commitment, uint256[] memory x, uint256 y)
        = _smallValidProof();

        for (uint256 j = 0; j < 256; ++j) {
            // Tweak
            y ^= 1 << j;
            vm.expectRevert();
            HyperKZGVerifier.__verifyHyperKZG({
                __proof: proof,
                __transcript: transcript,
                __commitment: commitment,
                __x: x,
                __y: y
            });
            // Untweak
            y ^= 1 << j;
        }
    }

    function testFuzzVerifyHyperKZGRevertsWithRandomInputs(
        bytes memory proof,
        uint256[1] memory transcript,
        uint256[2] memory commitment,
        uint256[] memory x,
        uint256 y
    ) public {
        vm.expectRevert();
        HyperKZGVerifier.__verifyHyperKZG({
            __proof: proof,
            __transcript: transcript,
            __commitment: commitment,
            __x: x,
            __y: y
        });
    }
}
