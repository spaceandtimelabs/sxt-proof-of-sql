// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

library TestScript {
    function testWeCanImportYulFromAnotherFile() public pure {
        bytes memory point0 = hex"0000000000000000" hex"0000000000000000" hex"0000000000000000"
            hex"0000000000000002" hex"0000000000000000" hex"0000000000000000" hex"0000000000000000"
            hex"0000000000000005";
        uint256 length0 = 1;
        uint256 numVars0 = 2;
        uint256 modulus0 = 10007;
        uint256 result0;
        assembly {
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.sol
            function compute_truncated_lagrange_basis_sum(length, point, num_vars, modulus) -> result {}
            result0 := compute_truncated_lagrange_basis_sum(length0, add(point0, 32), numVars0, modulus0)
        }
        assert(result0 == 4);
    }
}
