// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol"; // solhint-disable-line no-global-import
import {Test} from "forge-std/Test.sol";

contract VerificationBuilder is Test {
    function allocateBuilder() internal pure returns (uint256 builderPtr0) {
        assembly {
            function allocate_builder() -> builder_ptr {
                builder_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(builder_ptr, BUILDER_SIZE))
            }
            builderPtr0 := allocate_builder()
        }
    }

    function setChallenges(uint256 builderPtr0, uint256 challengePtr, uint256 challengeLength) internal pure {
        assembly {
            function set_challenges(builder_ptr, challenge_ptr, challenge_length) {
                mstore(add(builder_ptr, CHALLENGE_HEAD_OFFSET), challenge_ptr)
                mstore(
                    add(builder_ptr, CHALLENGE_TAIL_OFFSET),
                    add(challenge_ptr, shl(WORD_SHIFT, sub(challenge_length, 1)))
                )
            }
            set_challenges(builderPtr0, challengePtr, challengeLength)
        }
    }

    function consumeChallenge(uint256 builderPtr0) public pure returns (uint256 value0) {
        assembly {
            function consume_challenge(builder_ptr) -> value {
                let head_ptr_ptr := add(builder_ptr, CHALLENGE_HEAD_OFFSET)
                let head_ptr := mload(head_ptr_ptr)
                if gt(head_ptr, mload(add(builder_ptr, CHALLENGE_TAIL_OFFSET))) {
                    mstore(0, TOO_FEW_CHALLENGES)
                    revert(0, 4)
                }
                value := mload(head_ptr)
                head_ptr := add(head_ptr, WORD_SIZE)
                mstore(head_ptr_ptr, head_ptr)
            }
            value0 := consume_challenge(builderPtr0)
        }
    }

    function setFinalRoundMLEs(uint256 builderPtr0, uint256 finalRoundMLEPtr, uint256 finalRoundMLELength)
        internal
        pure
    {
        assembly {
            function set_final_round_mles(builder_ptr, final_round_mle_ptr, final_round_mle_length) {
                mstore(add(builder_ptr, FINAL_ROUND_MLE_HEAD_OFFSET), final_round_mle_ptr)
                mstore(
                    add(builder_ptr, FINAL_ROUND_MLE_TAIL_OFFSET),
                    add(final_round_mle_ptr, shl(WORD_SHIFT, sub(final_round_mle_length, 1)))
                )
            }
            set_final_round_mles(builderPtr0, finalRoundMLEPtr, finalRoundMLELength)
        }
    }

    function consumeFinalRoundMLE(uint256 builderPtr0) public pure returns (uint256 value0) {
        assembly {
            function consume_final_round_mle(builder_ptr) -> value {
                let head_ptr_ptr := add(builder_ptr, FINAL_ROUND_MLE_HEAD_OFFSET)
                let head_ptr := mload(head_ptr_ptr)
                if gt(head_ptr, mload(add(builder_ptr, FINAL_ROUND_MLE_TAIL_OFFSET))) {
                    mstore(0, TOO_FEW_FINAL_ROUND_MLES)
                    revert(0, 4)
                }
                value := calldataload(head_ptr)
                head_ptr := add(head_ptr, WORD_SIZE)
                mstore(head_ptr_ptr, head_ptr)
            }
            value0 := consume_final_round_mle(builderPtr0)
        }
    }

    function setOneEvaluations(uint256 builderPtr0, uint256 oneEvaluationPtr, uint256 oneEvaluationLength)
        internal
        pure
    {
        assembly {
            function set_one_evaluations(builder_ptr, one_evaluation_ptr, one_evaluation_length) {
                mstore(add(builder_ptr, ONE_EVALUATION_HEAD_OFFSET), one_evaluation_ptr)
                mstore(
                    add(builder_ptr, ONE_EVALUATION_TAIL_OFFSET),
                    add(one_evaluation_ptr, shl(WORD_SHIFT, sub(one_evaluation_length, 1)))
                )
            }
            set_one_evaluations(builderPtr0, oneEvaluationPtr, oneEvaluationLength)
        }
    }

    function consumeOneEvaluation(uint256 builderPtr0) public pure returns (uint256 value0) {
        assembly {
            function consume_one_evaluation(builder_ptr) -> value {
                let head_ptr_ptr := add(builder_ptr, ONE_EVALUATION_HEAD_OFFSET)
                let head_ptr := mload(head_ptr_ptr)
                if gt(head_ptr, mload(add(builder_ptr, ONE_EVALUATION_TAIL_OFFSET))) {
                    mstore(0, TOO_FEW_ONE_EVALUATIONS)
                    revert(0, 4)
                }
                value := mload(head_ptr)
                head_ptr := add(head_ptr, WORD_SIZE)
                mstore(head_ptr_ptr, head_ptr)
            }
            value0 := consume_one_evaluation(builderPtr0)
        }
    }

    function setSubpolynomialMultipliers(
        uint256 builderPtr0,
        uint256 subpolynomialMultiplierPtr,
        uint256 subpolynomialMultiplierLength
    ) internal pure {
        assembly {
            function set_subpolynomial_multipliers(
                builder_ptr, subpolynomial_multiplier_ptr, subpolynomial_multiplier_length
            ) {
                mstore(add(builder_ptr, SUBPOLYNOMIAL_MULTIPLIER_HEAD_OFFSET), subpolynomial_multiplier_ptr)
                mstore(
                    add(builder_ptr, SUBPOLYNOMIAL_MULTIPLIER_TAIL_OFFSET),
                    add(subpolynomial_multiplier_ptr, shl(WORD_SHIFT, sub(subpolynomial_multiplier_length, 1)))
                )
            }
            set_subpolynomial_multipliers(builderPtr0, subpolynomialMultiplierPtr, subpolynomialMultiplierLength)
        }
    }

    function setSumcheckEvaluations(uint256 builderPtr0, uint256 entrywiseMultiplierEvaluation, uint256 maxDegree)
        internal
        pure
    {
        assembly {
            function set_sumcheck_evaluations(builder_ptr, entrywise_multiplier_evaluation, max_degree) {
                mstore(add(builder_ptr, SUMCHECK_EVALUATION_OFFSET), 0)
                mstore(add(builder_ptr, ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET), entrywise_multiplier_evaluation)
                mstore(add(builder_ptr, MAX_DEGREE_OFFSET), max_degree)
            }
            set_sumcheck_evaluations(builderPtr0, entrywiseMultiplierEvaluation, maxDegree)
        }
    }

    function produceZerosumSubpolynomial(uint256 builderPtr0, uint256 evaluation0, uint256 degree0) public pure {
        assembly {
            function produce_zerosum_subpolynomial(builder_ptr, evaluation, degree) {
                if gt(degree, mload(add(builder_ptr, MAX_DEGREE_OFFSET))) {
                    mstore(0, TOO_FEW_SUBPOLYNOMIAL_MULTIPLIERS)
                    revert(0, 4)
                }
                let head_ptr_ptr := add(builder_ptr, SUBPOLYNOMIAL_MULTIPLIER_HEAD_OFFSET)
                let head_ptr := mload(head_ptr_ptr)
                if gt(head_ptr, mload(add(builder_ptr, SUBPOLYNOMIAL_MULTIPLIER_TAIL_OFFSET))) {
                    mstore(0, TOO_FEW_SUBPOLYNOMIAL_MULTIPLIERS)
                    revert(0, 4)
                }
                let sumcheck_evaluation_ptr := add(builder_ptr, SUMCHECK_EVALUATION_OFFSET)
                mstore(
                    sumcheck_evaluation_ptr,
                    addmod(mload(sumcheck_evaluation_ptr), mulmod(evaluation, mload(head_ptr), MODULUS), MODULUS)
                )
                head_ptr := add(head_ptr, WORD_SIZE)
                mstore(head_ptr_ptr, head_ptr)
            }
            produce_zerosum_subpolynomial(builderPtr0, evaluation0, degree0)
        }
    }

    function produceIdentitySubpolynomial(uint256 builderPtr0, uint256 evaluation0, uint256 degree0) public pure {
        assembly {
            function produce_identity_subpolynomial(builder_ptr, evaluation, degree) {
                if gt(add(degree, 1), mload(add(builder_ptr, MAX_DEGREE_OFFSET))) {
                    mstore(0, TOO_FEW_SUBPOLYNOMIAL_MULTIPLIERS)
                    revert(0, 4)
                }
                let head_ptr_ptr := add(builder_ptr, SUBPOLYNOMIAL_MULTIPLIER_HEAD_OFFSET)
                let head_ptr := mload(head_ptr_ptr)
                if gt(head_ptr, mload(add(builder_ptr, SUBPOLYNOMIAL_MULTIPLIER_TAIL_OFFSET))) {
                    mstore(0, TOO_FEW_SUBPOLYNOMIAL_MULTIPLIERS)
                    revert(0, 4)
                }
                let sumcheck_evaluation_ptr := add(builder_ptr, SUMCHECK_EVALUATION_OFFSET)
                let entrywise_multipliers_evaluation_ptr := add(builder_ptr, ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET)
                mstore(
                    sumcheck_evaluation_ptr,
                    addmod(
                        mload(sumcheck_evaluation_ptr),
                        mulmod(
                            mulmod(evaluation, mload(head_ptr), MODULUS),
                            mload(entrywise_multipliers_evaluation_ptr),
                            MODULUS
                        ),
                        MODULUS
                    )
                )
                head_ptr := add(head_ptr, WORD_SIZE)
                mstore(head_ptr_ptr, head_ptr)
            }
            produce_identity_subpolynomial(builderPtr0, evaluation0, degree0)
        }
    }

    function setRangeLength(uint256 builderPtr0, uint256 rangeLength0) internal pure {
        assembly {
            function set_range_length(builder_ptr, range_length) {
                mstore(add(builder_ptr, RANGE_LENGTH_OFFSET), range_length)
            }
            set_range_length(builderPtr0, rangeLength0)
        }
    }

    function getRangeLength(uint256 builderPtr0) internal pure returns (uint256 rangeLength0) {
        assembly {
            function get_range_length(builder_ptr) -> range_length {
                range_length := mload(add(builder_ptr, RANGE_LENGTH_OFFSET))
            }
            rangeLength0 := get_range_length(builderPtr0)
        }
    }

    function setEntrywisePointMultipliers(uint256 builderPtr0, uint256 entrywisePointMultipliers0) internal pure {
        assembly {
            function set_entrywise_point_multipliers(builder_ptr, entrywise_point_multipliers) {
                mstore(add(builder_ptr, ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET), entrywise_point_multipliers)
            }
            set_entrywise_point_multipliers(builderPtr0, entrywisePointMultipliers0)
        }
    }

    function getEntrywisePointMultipliers(uint256 builderPtr0)
        internal
        pure
        returns (uint256 entrywisePointMultipliers0)
    {
        assembly {
            function get_entrywise_point_multipliers(builder_ptr) -> entrywise_point_multipliers {
                entrywise_point_multipliers := mload(add(builder_ptr, ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET))
            }
            entrywisePointMultipliers0 := get_entrywise_point_multipliers(builderPtr0)
        }
    }
    //--------TESTS--------

    function testAllocateBuilder() public pure {
        uint256 prevFreePtr;
        assembly {
            prevFreePtr := mload(FREE_PTR)
        }
        uint256 builderPtr = allocateBuilder();
        uint256 freePtr;
        assembly {
            freePtr := mload(FREE_PTR)
        }
        assert(builderPtr == prevFreePtr);
        assert(freePtr == prevFreePtr + BUILDER_SIZE);
    }

    function testSetChallenges() public pure {
        uint256 builderPtr = allocateBuilder();
        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        uint256[3] memory challenges = [uint256(1), 2, 3];
        uint256 challengesPtr;
        assembly {
            challengesPtr := challenges
        }

        setChallenges(builderPtr, challengesPtr, 3);

        assert(builder[CHALLENGE_HEAD_OFFSET >> WORD_SHIFT] == challengesPtr);
        assert(builder[CHALLENGE_TAIL_OFFSET >> WORD_SHIFT] == challengesPtr + (2 << WORD_SHIFT));
    }

    function testConsumeChallenge() public pure {
        uint256[4] memory data = [uint256(1000), 1001, 1002, 1003];
        uint256 dataPtr;
        assembly {
            dataPtr := data
        }
        uint256 builderPtr = allocateBuilder();
        setChallenges(builderPtr, dataPtr, 4);

        assert(consumeChallenge(builderPtr) == 1000);
        assert(consumeChallenge(builderPtr) == 1001);
        assert(consumeChallenge(builderPtr) == 1002);
        assert(consumeChallenge(builderPtr) == 1003);

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }
        assert(builder[CHALLENGE_HEAD_OFFSET >> WORD_SHIFT] == dataPtr + (4 << WORD_SHIFT));
        assert(builder[CHALLENGE_TAIL_OFFSET >> WORD_SHIFT] == dataPtr + (3 << WORD_SHIFT));
    }

    function testConsumeChallengeRevertsIfThereAreTooFewChallenges() public {
        uint256[4] memory data = [uint256(1000), 1001, 1002, 1003];
        uint256 dataPtr;
        assembly {
            dataPtr := data
        }
        uint256 builderPtr = allocateBuilder();
        setChallenges(builderPtr, dataPtr, 3);

        assert(consumeChallenge(builderPtr) == 1000);
        assert(consumeChallenge(builderPtr) == 1001);
        assert(consumeChallenge(builderPtr) == 1002);
        vm.expectRevert(bytes4(uint32(TOO_FEW_CHALLENGES >> 224)));
        consumeChallenge(builderPtr);
    }

    function testSetFinalRoundMLEs() public pure {
        uint256 builderPtr = allocateBuilder();
        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        uint256[2] memory finalRoundMLEs = [uint256(4), 5];
        uint256 finalRoundMLEsPtr;
        assembly {
            finalRoundMLEsPtr := finalRoundMLEs
        }

        setFinalRoundMLEs(builderPtr, finalRoundMLEsPtr, 2);

        assert(builder[FINAL_ROUND_MLE_HEAD_OFFSET >> WORD_SHIFT] == finalRoundMLEsPtr);
        assert(builder[FINAL_ROUND_MLE_TAIL_OFFSET >> WORD_SHIFT] == finalRoundMLEsPtr + (1 << WORD_SHIFT));
    }

    function _testConsumeFinalRoundMLE() public pure {
        uint256[4] memory data = [uint256(1000), 1001, 1002, 1003];
        uint256 dataPtr;
        assembly {
            dataPtr := data
        }
        uint256 builderPtr = allocateBuilder();
        setFinalRoundMLEs(builderPtr, dataPtr, 4);

        assert(consumeFinalRoundMLE(builderPtr) == 1000);
        assert(consumeFinalRoundMLE(builderPtr) == 1001);
        assert(consumeFinalRoundMLE(builderPtr) == 1002);
        assert(consumeFinalRoundMLE(builderPtr) == 1003);

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }
        assert(builder[FINAL_ROUND_MLE_HEAD_OFFSET >> WORD_SHIFT] == dataPtr + (4 << WORD_SHIFT));
        assert(builder[FINAL_ROUND_MLE_TAIL_OFFSET >> WORD_SHIFT] == dataPtr + (3 << WORD_SHIFT));
    }

    function _testConsumeFinalRoundMLERevertsIfThereAreTooFewChallenges() public {
        uint256[4] memory data = [uint256(1000), 1001, 1002, 1003];
        uint256 dataPtr;
        assembly {
            dataPtr := data
        }
        uint256 builderPtr = allocateBuilder();
        setFinalRoundMLEs(builderPtr, dataPtr, 3);

        assert(consumeFinalRoundMLE(builderPtr) == 1000);
        assert(consumeFinalRoundMLE(builderPtr) == 1001);
        assert(consumeFinalRoundMLE(builderPtr) == 1002);
        vm.expectRevert(bytes4(uint32(TOO_FEW_FINAL_ROUND_MLES >> 224)));
        consumeFinalRoundMLE(builderPtr);
    }

    function testSetOneEvaluations() public pure {
        uint256 builderPtr = allocateBuilder();
        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        uint256[4] memory oneEvaluations = [uint256(6), 7, 8, 9];
        uint256 oneEvaluationsPtr;
        assembly {
            oneEvaluationsPtr := oneEvaluations
        }

        setOneEvaluations(builderPtr, oneEvaluationsPtr, 4);

        assert(builder[ONE_EVALUATION_HEAD_OFFSET >> WORD_SHIFT] == oneEvaluationsPtr);
        assert(builder[ONE_EVALUATION_TAIL_OFFSET >> WORD_SHIFT] == oneEvaluationsPtr + (3 << WORD_SHIFT));
    }

    function testConsumeOneEvaluation() public pure {
        uint256[4] memory data = [uint256(1000), 1001, 1002, 1003];
        uint256 dataPtr;
        assembly {
            dataPtr := data
        }
        uint256 builderPtr = allocateBuilder();
        setOneEvaluations(builderPtr, dataPtr, 4);

        assert(consumeOneEvaluation(builderPtr) == 1000);
        assert(consumeOneEvaluation(builderPtr) == 1001);
        assert(consumeOneEvaluation(builderPtr) == 1002);
        assert(consumeOneEvaluation(builderPtr) == 1003);

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }
        assert(builder[ONE_EVALUATION_HEAD_OFFSET >> WORD_SHIFT] == dataPtr + (4 << WORD_SHIFT));
        assert(builder[ONE_EVALUATION_TAIL_OFFSET >> WORD_SHIFT] == dataPtr + (3 << WORD_SHIFT));
    }

    function testConsumeOneEvaluationRevertsIfThereAreTooFewEvaluations() public {
        uint256[4] memory data = [uint256(1000), 1001, 1002, 1003];
        uint256 dataPtr;
        assembly {
            dataPtr := data
        }
        uint256 builderPtr = allocateBuilder();
        setOneEvaluations(builderPtr, dataPtr, 3);

        assert(consumeOneEvaluation(builderPtr) == 1000);
        assert(consumeOneEvaluation(builderPtr) == 1001);
        assert(consumeOneEvaluation(builderPtr) == 1002);
        vm.expectRevert(bytes4(uint32(TOO_FEW_ONE_EVALUATIONS >> 224)));
        consumeOneEvaluation(builderPtr);
    }

    function testSetSubpolynomialMultipliers() public pure {
        uint256 builderPtr = allocateBuilder();
        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        uint256[2] memory multipliers = [uint256(10), 11];
        uint256 multipliersPtr;
        assembly {
            multipliersPtr := multipliers
        }

        setSubpolynomialMultipliers(builderPtr, multipliersPtr, 2);

        assert(builder[SUBPOLYNOMIAL_MULTIPLIER_HEAD_OFFSET >> WORD_SHIFT] == multipliersPtr);
        assert(builder[SUBPOLYNOMIAL_MULTIPLIER_TAIL_OFFSET >> WORD_SHIFT] == multipliersPtr + (1 << WORD_SHIFT));
    }

    function testSetSumcheckEvaluations() public pure {
        uint256 builderPtr = allocateBuilder();
        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        setSumcheckEvaluations(builderPtr, 12, 3);

        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == 0);
        assert(builder[ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET >> WORD_SHIFT] == 12);
        assert(builder[MAX_DEGREE_OFFSET >> WORD_SHIFT] == 3);
    }

    function testProduceZerosumSubpolynomial() public pure {
        uint256[4] memory multipliers = [uint256(2), 3, 4, 5];
        uint256 multipliersPtr;
        assembly {
            multipliersPtr := multipliers
        }
        uint256 builderPtr = allocateBuilder();
        setSubpolynomialMultipliers(builderPtr, multipliersPtr, 4);

        setSumcheckEvaluations(builderPtr, 0, 3);
        uint256 expectedEvaluation = 0;

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        produceZerosumSubpolynomial(builderPtr, 10, 3);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(10, 2, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceZerosumSubpolynomial(builderPtr, 20, 3);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(20, 3, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceZerosumSubpolynomial(builderPtr, 30, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(30, 4, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceZerosumSubpolynomial(builderPtr, 40, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(40, 5, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);
        assert(builder[SUBPOLYNOMIAL_MULTIPLIER_HEAD_OFFSET >> WORD_SHIFT] == multipliersPtr + (4 << WORD_SHIFT));
        assert(builder[SUBPOLYNOMIAL_MULTIPLIER_TAIL_OFFSET >> WORD_SHIFT] == multipliersPtr + (3 << WORD_SHIFT));
    }

    function testProduceZerosumSubpolynomialRevertsIfThereAreTooFewMultipliers() public {
        uint256[4] memory multipliers = [uint256(2), 3, 4, 5];
        uint256 multipliersPtr;
        assembly {
            multipliersPtr := multipliers
        }
        uint256 builderPtr = allocateBuilder();
        setSubpolynomialMultipliers(builderPtr, multipliersPtr, 3);

        setSumcheckEvaluations(builderPtr, 0, 3);

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        produceZerosumSubpolynomial(builderPtr, 10, 3);
        produceZerosumSubpolynomial(builderPtr, 20, 3);
        produceZerosumSubpolynomial(builderPtr, 30, 2);
        vm.expectRevert(bytes4(uint32(TOO_FEW_SUBPOLYNOMIAL_MULTIPLIERS >> 224)));
        produceZerosumSubpolynomial(builderPtr, 40, 2);
    }

    function testProduceIdentitySubpolynomial() public pure {
        uint256[4] memory multipliers = [uint256(2), 3, 4, 5];
        uint256 multipliersPtr;
        assembly {
            multipliersPtr := multipliers
        }
        uint256 builderPtr = allocateBuilder();
        setSubpolynomialMultipliers(builderPtr, multipliersPtr, 4);

        setSumcheckEvaluations(builderPtr, 7, 3);
        uint256 expectedEvaluation = 0;

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        produceIdentitySubpolynomial(builderPtr, 10, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(mulmod(10, 2, MODULUS), 7, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceIdentitySubpolynomial(builderPtr, 20, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(mulmod(20, 3, MODULUS), 7, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceIdentitySubpolynomial(builderPtr, 30, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(mulmod(30, 4, MODULUS), 7, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceIdentitySubpolynomial(builderPtr, 40, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(mulmod(40, 5, MODULUS), 7, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);
        assert(builder[SUBPOLYNOMIAL_MULTIPLIER_HEAD_OFFSET >> WORD_SHIFT] == multipliersPtr + (4 << WORD_SHIFT));
        assert(builder[SUBPOLYNOMIAL_MULTIPLIER_TAIL_OFFSET >> WORD_SHIFT] == multipliersPtr + (3 << WORD_SHIFT));
        assert(builder[ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET >> WORD_SHIFT] == 7);
    }

    function testProduceIdentitySubpolynomialRevertsIfThereAreTooFewMultipliers() public {
        uint256[4] memory multipliers = [uint256(2), 3, 4, 5];
        uint256 multipliersPtr;
        assembly {
            multipliersPtr := multipliers
        }
        uint256 builderPtr = allocateBuilder();
        setSubpolynomialMultipliers(builderPtr, multipliersPtr, 3);

        setSumcheckEvaluations(builderPtr, 7, 3);

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        produceIdentitySubpolynomial(builderPtr, 10, 2);
        produceIdentitySubpolynomial(builderPtr, 20, 2);
        produceIdentitySubpolynomial(builderPtr, 30, 2);
        vm.expectRevert(bytes4(uint32(TOO_FEW_SUBPOLYNOMIAL_MULTIPLIERS >> 224)));
        produceIdentitySubpolynomial(builderPtr, 40, 2);
    }

    function testProduceMixedSubpolynomial() public pure {
        uint256[4] memory multipliers = [uint256(2), 3, 4, 5];
        uint256 multipliersPtr;
        assembly {
            multipliersPtr := multipliers
        }
        uint256 builderPtr = allocateBuilder();
        setSubpolynomialMultipliers(builderPtr, multipliersPtr, 4);

        setSumcheckEvaluations(builderPtr, 7, 3);
        uint256 expectedEvaluation = 0;

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] = expectedEvaluation;
        builder[ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET >> WORD_SHIFT] = 7;

        produceZerosumSubpolynomial(builderPtr, 10, 3);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(10, 2, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceIdentitySubpolynomial(builderPtr, 20, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(mulmod(20, 3, MODULUS), 7, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceZerosumSubpolynomial(builderPtr, 30, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(30, 4, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        produceIdentitySubpolynomial(builderPtr, 40, 2);
        expectedEvaluation = addmod(expectedEvaluation, mulmod(mulmod(40, 5, MODULUS), 7, MODULUS), MODULUS);
        assert(builder[SUMCHECK_EVALUATION_OFFSET >> WORD_SHIFT] == expectedEvaluation);

        assert(builder[SUBPOLYNOMIAL_MULTIPLIER_HEAD_OFFSET >> WORD_SHIFT] == multipliersPtr + (4 << WORD_SHIFT));
        assert(builder[SUBPOLYNOMIAL_MULTIPLIER_TAIL_OFFSET >> WORD_SHIFT] == multipliersPtr + (3 << WORD_SHIFT));
        assert(builder[ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET >> WORD_SHIFT] == 7);
    }

    function testProduceMixedSubpolynomialRevertsIfThereAreTooFewMultipliers() public {
        uint256[4] memory multipliers = [uint256(2), 3, 4, 5];
        uint256 multipliersPtr;
        assembly {
            multipliersPtr := multipliers
        }
        uint256 builderPtr = allocateBuilder();
        setSubpolynomialMultipliers(builderPtr, multipliersPtr, 3);

        setSumcheckEvaluations(builderPtr, 7, 3);

        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        produceZerosumSubpolynomial(builderPtr, 10, 3);
        produceIdentitySubpolynomial(builderPtr, 20, 2);
        produceZerosumSubpolynomial(builderPtr, 30, 2);
        vm.expectRevert(bytes4(uint32(TOO_FEW_SUBPOLYNOMIAL_MULTIPLIERS >> 224)));
        produceIdentitySubpolynomial(builderPtr, 40, 2);
    }

    function testSetAndGetRangeLength() public pure {
        uint256 builderPtr = allocateBuilder();
        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        setRangeLength(builderPtr, 123);

        assert(builder[RANGE_LENGTH_OFFSET >> WORD_SHIFT] == 123);
        assert(getRangeLength(builderPtr) == 123);
    }

    function testSetAndGetEntrywisePointMultipliers() public pure {
        uint256 builderPtr = allocateBuilder();
        uint256[BUILDER_SIZE >> WORD_SHIFT] memory builder;
        assembly {
            builder := builderPtr
        }

        setEntrywisePointMultipliers(builderPtr, 123);

        assert(builder[ENTRYWISE_MULTIPLIERS_EVALUATION_OFFSET >> WORD_SHIFT] == 123);
        assert(getEntrywisePointMultipliers(builderPtr) == 123);
    }
}
