use syn::{Attribute, Meta};

/// Macro for running setup routines
/// when the test binary first starts.
pub(crate) mod setup;

/// Macro for single tests.
pub(crate) mod test;

/// Macro for test suites.
pub(crate) mod tests;

// Helper function to pull the string out of #[doc = "..."]
pub(crate) fn extract_doc_string(attr: &Attribute) -> Option<String> {
    if let Meta::NameValue(name_value) = &attr.meta {
        if name_value.path.is_ident("doc") {
            if let syn::Expr::Lit(expr_lit) = &name_value.value {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}
