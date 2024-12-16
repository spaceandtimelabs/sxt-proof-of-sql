/// Construct an `Ident` from a string.
#[cfg(test)]
pub(crate) fn ident(name: &str) -> sqlparser::ast::Ident {
    name.into()
}
