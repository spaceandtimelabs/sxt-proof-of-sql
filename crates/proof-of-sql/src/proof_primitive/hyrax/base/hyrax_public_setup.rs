/// The proof and verification setup for the Hyrax scheme
#[derive(Clone, Copy)]
pub struct HyraxPublicSetup<'a, G>
where
    for<'b> G: 'b,
{
    /// The generators used to generate `HyraxCommitment`s.
    pub generators: &'a [G],
}
