/// Track components used to form a query's proof
#[allow(dead_code)]
pub struct ProofBuilder<'a> {
    dummy: std::marker::PhantomData<&'a ()>, // will be replaced with non-owning data structures referencing arena
                                             // allocated values
}
