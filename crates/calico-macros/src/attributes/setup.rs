use proc_macro::TokenStream;
use proc_macro_error2::{abort, abort_call_site};
use syn::{ItemFn, ReturnType, parse_macro_input};
use quote::quote;

/// Expand the proc_macro_attribute parameters into a setup function.
pub(crate) fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        abort_call_site!("`#[calico::setup]` attribute takes no arguments");
    }

    // Parse the TokenStream into a syn AST node.
    let input_fn = parse_macro_input!(input as ItemFn);

    // Validate the structure of the function.
    if input_fn.sig.constness.is_some() // must not be const
        // must not be async
        || input_fn.sig.asyncness.is_some() 
        // must not be unsafe
        || input_fn.sig.unsafety.is_some() 
        // must not be ABI
        || input_fn.sig.abi.is_some() 
        // must not take generics
        || !input_fn.sig.generics.params.is_empty() 
        // must not have generics `where` clause
        || input_fn.sig.generics.where_clause.is_some() 
        // must not be variadic
        || input_fn.sig.variadic.is_some()
        // must not take inputs
        || !input_fn.sig.inputs.is_empty() 
        // must not return anything
        || input_fn.sig.output != ReturnType::Default
    {
        abort!(input_fn.sig.ident, "function must have signature `fn() -> () `");
    }

    // Validate that attributes that break discovering the setup function aren't present.
    //
    // This is important because the macro injects these attributes to make sure that the
    // setup function is named correctly to be discovered by the test harness entrypoints.
    let reject_list = &["export_name", "no_mangle"];
    for attr in &input_fn.attrs {
        if let Some(ident) = attr.path().get_ident() {
            let ident = ident.to_string();

            if reject_list.contains(&ident.as_str()) {
                abort!(
                    attr,
                    "`#[setup]` attribute cannot be used together with `#[{}]`",
                    ident
                )
            }
        }
    }

    let attrs = &input_fn.attrs;
    let block = &input_fn.block;
    let ident = &input_fn.sig.ident;

    // Annotate the provided setup function with export_name so that the
    // test harness entrypoints can correctly discover and call it.
    //
    // The way this is written also re-forms the function signature using
    // the function name and body, which will cause a compiler error if the
    // function signature is not correct and incorrectly includes parameters.
    quote!(
        #(#attrs)*
        #[unsafe(export_name = "_calico_setup")]
        #[inline(never)]
        fn #ident() {
            #block
        }
    )
    .into()
}
