//! Module holding the `RefInto` trait.

/// A reference-to-value conversion that does not consume the input value.
///
/// This is automatically implemented for all `S` where `&S` implements `Into<T>`.
///
/// This is primarily useful when defining subtraits. For example, here is a trait that requires
/// the implementation of conversions to and from `usize` for both values and references:
/// ```ignore
/// pub trait SubTrait: From<usize> + Into<usize> + for<'a> From<&'a usize> + RefInto<usize> {
///     ...
/// }
/// ```
pub trait RefInto<T> {
    /// Converts a reference to this type into the (usually inferred) input type.
    fn ref_into(&self) -> T;
}
impl<T, S> RefInto<T> for S
where
    for<'a> &'a S: Into<T>,
{
    fn ref_into(&self) -> T {
        self.into()
    }
}
