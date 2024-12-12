use sqlparser::ast::Ident;

// Normalize an owned identifier to a lowercase string unless the identifier is quoted.
pub(crate) fn normalize_ident(id: Ident) -> alloc::string::String {
    match id.quote_style {
        Some(_) => id.value,
        None => id.value.to_ascii_lowercase(),
    }
}

/// Construct an `Ident` from a string.
#[cfg(test)]
pub(crate) fn ident(name: &str) -> Ident {
    name.into()
}
