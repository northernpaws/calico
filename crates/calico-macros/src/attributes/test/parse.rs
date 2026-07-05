use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;

/// Defines the arguments that can be supplied
/// to the `test` macro when invoked.
#[derive(Debug, FromMeta)]
pub(crate) struct MacroArgs {
    /// Optionally set the camel_case name of the test.
    pub name: Option<String>,

    /// An optional, short, human-friendly, label
    /// displayed by the test runner to the end user.
    ///
    /// This label is not compiled into the binary,
    /// and is only stored in metadata read by the
    /// test runner.
    #[cfg(feature = "metadata")]
    pub label: Option<String>,

    /// An optional extended description describing the test.
    ///
    /// If there is a doc comment above the test, then that
    /// will be automatically injested as the description.
    #[cfg(feature = "metadata")]
    pub description: Option<String>,
}

impl MacroArgs {
    /// Parse a set of macro arguments from the supplied arguments token stream.
    pub(crate) fn parse(args: TokenStream) -> Result<Self, syn::Error> {
        let attr_args = NestedMeta::parse_meta_list(args.into())?;
        let macro_args = MacroArgs::from_list(&attr_args)?;
        Ok(macro_args)
    }
}
