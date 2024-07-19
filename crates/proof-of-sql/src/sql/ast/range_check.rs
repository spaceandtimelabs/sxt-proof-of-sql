use crate::{
    base::{commitment::Commitment, proof::ProofError, scalar::Scalar},
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use core::slice;

/**

range check node

decompose scalar to 248 / 8

looks like equals expr, we only need lhs

take scalar

break into 31 bytes

produce proofs that each byte in column is in range from 2^248

count_range_check
verify_evaluate_range_check
proof_evaluate_range_check
result_evaluate_range_check

return range check but dont need to return, just result

produce intermediate byte columns
produce polynomials
verify and consume polynomials


dont need byte distribution complexity
break into 31 bytes
get columns -> intermediate mles
*/

///
pub fn count_range_check(builder: &mut CountBuilder) -> Result<(), ProofError> {
    todo!()
}

///
pub fn result_evaluate_range_check<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    let results: Vec<[u64; 4]> = expr.iter().map(|&s| s.into()).collect();

    // Create a slice of bytes that spans all `[u64; 4]` entries
    let bytes: &[u8] = {
        let ptr = results.as_ptr() as *const u8;
        let len = results.len() * std::mem::size_of::<[u64; 4]>();
        unsafe { slice::from_raw_parts(ptr, len) }
    };


    &[true]  // Assuming the original return type of &[bool] needs to be maintained
}

///
pub fn prover_evaluate_range_check<'a, S: Scalar>(
    builder: &mut ProofBuilder<'a, S>,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    todo!()
}

///
pub fn verifier_evaluate_range_check<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    eval: C::Scalar,
    one_eval: C::Scalar,
) -> Result<C::Scalar, ProofError> {
    todo!()
}
