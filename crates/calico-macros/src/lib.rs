//! Provides macros used by Calico to emit the
//! instrumentation files used by the test runner.

extern crate proc_macro;
use proc_macro::TokenStream;

mod attributes;

/// Annotates a module of functions as a test suite.
#[proc_macro_attribute]
pub fn tests(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::tests::expand(args, input)
}

/// Annotates a function for running an isolated test,
/// similar to Rust's native #[test] attribute.
///
/// Attributes:
/// - `name` - Sets the test name instead of the function name.
#[proc_macro_attribute]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::test::expand(args, input)
}
