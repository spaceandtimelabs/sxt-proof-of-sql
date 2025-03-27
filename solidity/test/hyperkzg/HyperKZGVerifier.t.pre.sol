// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import "../../src/base/Errors.sol";
import {HyperKZGVerifier} from "../../src/hyperkzg/HyperKZGVerifier.pre.sol";

contract HyperKZGVerifierTest is Test {
    function verifyHyperKZG(
        bytes calldata proof,
        uint256[1] memory transcript,
        uint256[2] memory commitment,
        uint256[] memory x,
        uint256 y
    ) public view {
        HyperKZGVerifier.__verifyHyperKZG({
            __proof: proof,
            __transcript: transcript,
            __commitment: commitment,
            __x: x,
            __y: y
        });
    }

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
        proof = hex"0000000000000001" hex"1f2e45337f9b8344112089d02a9827c05864124c9d68e6dfe4ae4b1ef18b8bec"
            hex"0603731e181537ca4cac7f28123622a175a7181b08404a64b1197c3a8adee75c" hex"0000000000000002"
            hex"195601834abe3b06307843dfb2bda53c463acac5ce7452fe7f9afb76ef076159"
            hex"01b06ce4e5c076c62ee49bea6c8c0d3475c9c4203863f33e407c032f104d7929"
            hex"0244bf82e008d1628941372c47440cf0b7ddf46a4e07f370e99246946d2c2f96"
            hex"1fac3ee761eb0bf01d1be7d167923af7a75802fb6daaf7c93b17e48bdb93582a"
            hex"10b80f8b7f4694399b345de519ef1d6580dbe54d0c0e78c808ca1108146ca7e5"
            hex"249c5130fc843ff6fa68d4ab85a5254f12f04d615289e5c00e42c22b867eebab"
            hex"20aaca7d9451a200e417af702479ee0bd90a19d25f90bc5ac31515a2510ecbf9"
            hex"13880503efe944d65ff45424e0c07237dbb4bbfa7d8dcbed7c0f234d94afd8c0"
            hex"0afe9d625909d59d10259307138122091a2a4d81d7e419359e9a3b70916edade"
            hex"12f9228a1c7fa913c0e3b4b20ff5cf106c9555c20c668e6441dfb3fa1a174626"
            hex"18977d28d54a74822b9816495ab7909d9db911b3d107a7ccbb758baa94217fd5"
            hex"097467f5beafcf7a6b77c515a0876800db96837e5e279cd8a0d3f86deb571fb7";
        transcript = [0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470];
        commitment = [
            0x0bceea30108fed3f7c8e53e56a3aedf0de0bc26292e469ab525f1ac9fe93c758,
            0x126f299ba3b83331a6901a281b0982b87f154efe57ac4da1f7c51a97dda59e1b
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
        uint256 vOffset = WORDX2_SIZE * ell - WORDX2_SIZE + UINT64_SIZE * 2;

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
