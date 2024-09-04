use crate::base::{commitment::Commitment, database::Column};

/// Result of evaluation of `ProofExpr`
#[derive(Clone, Debug)]
pub struct ProofExprResult<'a, C: Commitment> {
    /// Evaluation result of the expression itself
    pub result: Column<'a, C::Scalar>,
    /// `ProofExprResult` of its child expressions
    pub children: Vec<Box<ProofExprResult<'a, C>>>,
}

impl<'a, C: Commitment> ProofExprResult<'a, C> {
    /// Create a new `ProofExprResult`
    pub fn new(result: Column<'a, C::Scalar>, children: Vec<Box<ProofExprResult<'a, C>>>) -> Self {
        Self { result, children }
    }
}
