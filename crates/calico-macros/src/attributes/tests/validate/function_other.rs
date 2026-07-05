use proc_macro_error2::abort;

use crate::attributes::tests::parse::ModuleFn;

/// Represents a function parsed in a test
/// module that's not a known test function.
pub(crate) struct OtherFunc(pub ModuleFn);

impl From<ModuleFn> for OtherFunc {
    fn from(func: ModuleFn) -> Self {
        if let Some((_attr, span)) = func.attributes.first() {
            abort!(
                span,
                "Only `#[test]` or `#[init]` functions can have such an attribute"
            );
        }
        OtherFunc(func)
    }
}
