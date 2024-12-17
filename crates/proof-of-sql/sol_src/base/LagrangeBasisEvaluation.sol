// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract LagrangeBasisEvaluation {
    function computeTruncatedLagrangeBasisSum(uint256 length0, bytes memory point0, uint256 numVars0, uint256 modulus0)
        public
        pure
        returns (uint256 result0)
    {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            // START-YUL compute_truncated_lagrange_basis_sum
            function compute_truncated_lagrange_basis_sum(length, point, num_vars, modulus) -> result {
                let ONE := add(modulus, 1)
                // result := 0 // implicitly set by the EVM

                // Invariant that holds within the for loop:
                // 0 <= result <= modulus + 1
                // This invariant reduces modulus operations.
                for {} num_vars {} {
                    switch and(length, 1)
                    case 0 { result := mulmod(result, sub(ONE, mod(mload(point), modulus)), modulus) }
                    default { result := sub(ONE, mulmod(sub(ONE, result), mload(point), modulus)) }
                    num_vars := sub(num_vars, 1)
                    length := shr(1, length)
                    point := add(point, 32)
                }
                switch length
                case 0 { result := mod(result, modulus) }
                default { result := 1 }
            }
            // END-YUL
            result0 := compute_truncated_lagrange_basis_sum(length0, add(point0, 32), numVars0, modulus0)
        }
    }

    uint256 private constant TEST_MODULUS = 10007;

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith0Variables() public pure {
        bytes memory point = hex"";
        assert(computeTruncatedLagrangeBasisSum(1, point, 0, TEST_MODULUS) == 1);
        assert(computeTruncatedLagrangeBasisSum(0, point, 0, TEST_MODULUS) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith1Variables() public pure {
        bytes memory point = hex"0000000000000000" hex"0000000000000000" hex"0000000000000000" hex"0000000000000002";
        assert(computeTruncatedLagrangeBasisSum(2, point, 1, TEST_MODULUS) == 1);
        assert(computeTruncatedLagrangeBasisSum(1, point, 1, TEST_MODULUS) == TEST_MODULUS - 1);
        assert(computeTruncatedLagrangeBasisSum(0, point, 1, TEST_MODULUS) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith2Variables() public pure {
        bytes memory point = hex"0000000000000000" hex"0000000000000000" hex"0000000000000000" hex"0000000000000002"
            hex"0000000000000000" hex"0000000000000000" hex"0000000000000000" hex"0000000000000005";
        assert(computeTruncatedLagrangeBasisSum(4, point, 2, TEST_MODULUS) == 1);
        assert(computeTruncatedLagrangeBasisSum(3, point, 2, TEST_MODULUS) == TEST_MODULUS - 9);
        assert(computeTruncatedLagrangeBasisSum(2, point, 2, TEST_MODULUS) == TEST_MODULUS - 4);
        assert(computeTruncatedLagrangeBasisSum(1, point, 2, TEST_MODULUS) == 4);
        assert(computeTruncatedLagrangeBasisSum(0, point, 2, TEST_MODULUS) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith3Variables() public pure {
        bytes memory point = hex"0000000000000000" hex"0000000000000000" hex"0000000000000000" hex"0000000000000002"
            hex"0000000000000000" hex"0000000000000000" hex"0000000000000000" hex"0000000000000005"
            hex"0000000000000000" hex"0000000000000000" hex"0000000000000000" hex"0000000000000007";
        assert(computeTruncatedLagrangeBasisSum(8, point, 3, TEST_MODULUS) == 1);
        assert(computeTruncatedLagrangeBasisSum(7, point, 3, TEST_MODULUS) == TEST_MODULUS - 69);
        assert(computeTruncatedLagrangeBasisSum(6, point, 3, TEST_MODULUS) == TEST_MODULUS - 34);
        assert(computeTruncatedLagrangeBasisSum(5, point, 3, TEST_MODULUS) == 22);
        assert(computeTruncatedLagrangeBasisSum(4, point, 3, TEST_MODULUS) == TEST_MODULUS - 6);
        assert(computeTruncatedLagrangeBasisSum(3, point, 3, TEST_MODULUS) == 54);
        assert(computeTruncatedLagrangeBasisSum(2, point, 3, TEST_MODULUS) == 24);
        assert(computeTruncatedLagrangeBasisSum(1, point, 3, TEST_MODULUS) == TEST_MODULUS - 24);
        assert(computeTruncatedLagrangeBasisSum(0, point, 3, TEST_MODULUS) == 0);
    }
}
