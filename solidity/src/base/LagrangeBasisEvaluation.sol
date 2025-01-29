// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "./Constants.sol"; // solhint-disable-line no-global-import

library LagrangeBasisEvaluation {
    function computeTruncatedLagrangeBasisSum(uint256 length_, uint256 pointPtr_, uint256 numVars_)
        public
        pure
        returns (uint256 result0)
    {
        assembly {
            function compute_truncated_lagrange_basis_sum(length, point_ptr, num_vars) -> result {
                result := 0

                // Invariant that holds within the for loop:
                // 0 <= result <= modulus + 1
                // This invariant reduces modulus operations.
                for {} num_vars {} {
                    switch and(length, 1)
                    case 0 { result := mulmod(result, sub(MODULUS_PLUS_ONE, mod(mload(point_ptr), MODULUS)), MODULUS) }
                    default {
                        result :=
                            sub(MODULUS_PLUS_ONE, mulmod(sub(MODULUS_PLUS_ONE, result), mload(point_ptr), MODULUS))
                    }
                    num_vars := sub(num_vars, 1)
                    length := shr(1, length)
                    point_ptr := add(point_ptr, WORD_SIZE)
                }
                switch length
                case 0 { result := mod(result, MODULUS) }
                default { result := 1 }
            }
            result0 := compute_truncated_lagrange_basis_sum(length_, pointPtr_, numVars_)
        }
    }

    function computeTruncatedLagrangeBasisInnerProduct(uint256 length_, uint256 aPtr_, uint256 bPtr_, uint256 numVars_)
        public
        pure
        returns (uint256 result0)
    {
        assembly {
            function compute_truncated_lagrange_basis_inner_product(length, a_ptr, b_ptr, num_vars) -> result {
                let part := 0
                result := 1
                for { let i := 0 } sub(num_vars, i) { i := add(i, 1) } {
                    let a := mload(a_ptr)
                    let b := mload(b_ptr)
                    a_ptr := add(a_ptr, WORD_SIZE)
                    b_ptr := add(b_ptr, WORD_SIZE)
                    let ab := mulmod(a, b, MODULUS)
                    let cd := sub(add(MODULUS_PLUS_ONE, ab), addmod(a, b, MODULUS))
                    switch and(shr(i, length), 1)
                    case 0 { part := mulmod(part, cd, MODULUS) }
                    default { part := add(mulmod(result, cd, MODULUS), mulmod(part, ab, MODULUS)) }
                    result := mulmod(result, add(cd, ab), MODULUS)
                }
                if lt(length, shl(num_vars, 1)) { result := mod(part, MODULUS) }
            }
            result0 := compute_truncated_lagrange_basis_inner_product(length_, aPtr_, bPtr_, numVars_)
        }
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith0Variables() public pure {
        uint256[1] memory point;
        uint256 pointPtr;
        assembly {
            pointPtr := point
        }
        assert(computeTruncatedLagrangeBasisSum(1, pointPtr, 0) == 1);
        assert(computeTruncatedLagrangeBasisSum(0, pointPtr, 0) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith1Variables() public pure {
        uint256[1] memory point = [uint256(2)];
        uint256 pointPtr;
        assembly {
            pointPtr := point
        }
        assert(computeTruncatedLagrangeBasisSum(2, pointPtr, 1) == 1);
        assert(computeTruncatedLagrangeBasisSum(1, pointPtr, 1) == MODULUS - 1);
        assert(computeTruncatedLagrangeBasisSum(0, pointPtr, 1) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith2Variables() public pure {
        uint256[2] memory point = [uint256(2), 5];
        uint256 pointPtr;
        assembly {
            pointPtr := point
        }
        assert(computeTruncatedLagrangeBasisSum(4, pointPtr, 2) == 1);
        assert(computeTruncatedLagrangeBasisSum(3, pointPtr, 2) == MODULUS - 9);
        assert(computeTruncatedLagrangeBasisSum(2, pointPtr, 2) == MODULUS - 4);
        assert(computeTruncatedLagrangeBasisSum(1, pointPtr, 2) == 4);
        assert(computeTruncatedLagrangeBasisSum(0, pointPtr, 2) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith3Variables() public pure {
        uint256[3] memory point = [uint256(2), 5, 7];
        uint256 pointPtr;
        assembly {
            pointPtr := point
        }
        assert(computeTruncatedLagrangeBasisSum(8, pointPtr, 3) == 1);
        assert(computeTruncatedLagrangeBasisSum(7, pointPtr, 3) == MODULUS - 69);
        assert(computeTruncatedLagrangeBasisSum(6, pointPtr, 3) == MODULUS - 34);
        assert(computeTruncatedLagrangeBasisSum(5, pointPtr, 3) == 22);
        assert(computeTruncatedLagrangeBasisSum(4, pointPtr, 3) == MODULUS - 6);
        assert(computeTruncatedLagrangeBasisSum(3, pointPtr, 3) == 54);
        assert(computeTruncatedLagrangeBasisSum(2, pointPtr, 3) == 24);
        assert(computeTruncatedLagrangeBasisSum(1, pointPtr, 3) == MODULUS - 24);
        assert(computeTruncatedLagrangeBasisSum(0, pointPtr, 3) == 0);
    }

    function testComputeTruncatedLagrangeBasisInnerProductGivesCorrectValuesWith0Variables() public pure {
        uint256[1] memory a;
        uint256[1] memory b;
        uint256 aPtr;
        uint256 bPtr;
        assembly {
            aPtr := a
            bPtr := b
        }
        assert(computeTruncatedLagrangeBasisInnerProduct(1, aPtr, bPtr, 0) == 1);
        assert(computeTruncatedLagrangeBasisInnerProduct(0, aPtr, bPtr, 0) == 0);
    }

    function testComputeTruncatedLagrangeBasisInnerProductGivesCorrectValuesWith1Variable() public pure {
        uint256[1] memory a = [uint256(2)];
        uint256[1] memory b = [uint256(3)];
        uint256 aPtr;
        uint256 bPtr;
        assembly {
            aPtr := a
            bPtr := b
        }
        assert(computeTruncatedLagrangeBasisInnerProduct(2, aPtr, bPtr, 1) == 8);
        assert(computeTruncatedLagrangeBasisInnerProduct(1, aPtr, bPtr, 1) == 2);
        assert(computeTruncatedLagrangeBasisInnerProduct(0, aPtr, bPtr, 1) == 0);
    }

    function testComputeTruncatedLagrangeBasisInnerProductGivesCorrectValuesWith3Variables() public pure {
        uint256[3] memory a = [uint256(2), 5, 7];
        uint256[3] memory b = [uint256(3), 11, 13];
        uint256 aPtr;
        uint256 bPtr;
        assembly {
            aPtr := a
            bPtr := b
        }
        assert(computeTruncatedLagrangeBasisInnerProduct(8, aPtr, bPtr, 3) == 123880);
        assert(computeTruncatedLagrangeBasisInnerProduct(7, aPtr, bPtr, 3) == 93850);
        assert(computeTruncatedLagrangeBasisInnerProduct(6, aPtr, bPtr, 3) == 83840);
        assert(computeTruncatedLagrangeBasisInnerProduct(5, aPtr, bPtr, 3) == 62000);
        assert(computeTruncatedLagrangeBasisInnerProduct(4, aPtr, bPtr, 3) == 54720);
        assert(computeTruncatedLagrangeBasisInnerProduct(3, aPtr, bPtr, 3) == 30960);
        assert(computeTruncatedLagrangeBasisInnerProduct(2, aPtr, bPtr, 3) == 23040);
        assert(computeTruncatedLagrangeBasisInnerProduct(1, aPtr, bPtr, 3) == 5760);
        assert(computeTruncatedLagrangeBasisInnerProduct(0, aPtr, bPtr, 3) == 0);
    }
}
