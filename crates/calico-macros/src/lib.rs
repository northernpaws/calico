//! Provides macros used by Calico to emit the
//! instrumentation files used by the test runner.

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

mod attributes;

/// Annotates a module of functions as a test suite.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn tests(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::tests::expand(args, input)
}

/// Annotates a function for running an isolated test,
/// similar to Rust's native #[test] attribute.
///
/// Attributes:
/// - `name` - Sets the test name instead of the function name.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::test::expand(args, input)
}

/// Annotates a global setup function for a test suite.
///
/// ## Examples
///
/// ```rust,no_run
/// #[cfg(test)]
/// #[calico::setup]
/// fn setup() {
///     ... setup code ...
/// }
/// ```
///
#[proc_macro_attribute]
#[proc_macro_error]
pub fn setup(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::setup::expand(args, input)
}
