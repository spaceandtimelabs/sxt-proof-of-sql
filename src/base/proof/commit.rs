use impl_trait_for_tuples::impl_for_tuples;

/// Indicates that a type has an associated commitment type, and provides a method for calculating
/// the commitment.
///
/// A notable implementation of this type is on tuples of other Commit types.
pub trait Commit {
    type Commitment;

    /// Calculate the associated commitment type, [Self::Commitment].
    ///
    /// # Security
    /// To avoid security defects, the commited type should never be updated post-commitment.
    /// In the future, it may be plausible to enforce this with the type system.
    fn commit(&self) -> Self::Commitment;
}

#[allow(clippy::unused_unit)]
#[impl_for_tuples(5)]
impl Commit for Tuple {
    for_tuples!( type Commitment = ( #( Tuple::Commitment ),* ); );
    fn commit(&self) -> Self::Commitment {
        for_tuples!(( #( Tuple.commit() ),* ));
    }
}
