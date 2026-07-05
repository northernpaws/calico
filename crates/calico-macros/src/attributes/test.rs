//! Proc macro attribute for flagging a function as a test.

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro_error2::abort;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

mod parse;

/// Expand the proc_macro_attribute parameters into a functional test.
pub(crate) fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the TokenStream into a syn AST node.
    // let input_fn: TestFunc = parse_macro_input!(input as TestFunc);
    let input_fn = parse_macro_input!(input as ItemFn);

    let macro_args = match parse::MacroArgs::parse(args) {
        Ok(args) => args,
        Err(e) => abort!(e),
    };

    // Parse the macro attributes as a comma seperate list.
    // let attrs: Punctuated<Meta, syn::token::Comma> =
    //     parse_macro_input!(attr with Punctuated::<Meta, syn::Token![,]>::parse_terminated);

    match test_inner(input_fn, macro_args) {
        Ok(output) => output.into(),
        // Convert a syn error into a sequence that displays it nicely in the compiler.
        Err(err) => err.to_compile_error().into(),
    }
}

/// Allows for using Rust Result syntax for handling syn errors.
fn test_inner(mut input_fn: ItemFn, args: parse::MacroArgs) -> Result<TokenStream, syn::Error> {
    // Extract the function name for use as the test name/identifier.
    let function_ident = &input_fn.sig.ident;
    let function_name_string = function_ident.to_string();

    // Attributes about the test that we need
    // to try to determine or parse next.
    let mut test_name = function_name_string;

    // Parse the attributes supplies as arguments to the macro.
    if let Some(name) = args.name {
        test_name = name;
    }

    // Set by rustc to a directory relative to the target directory, such as:
    // calico/examples/stm32f401re/target/thumbv7em-none-eabi/debug/build/stm32f401re-a1f4896cb8b22f9a/out
    //
    // There is also:
    // CARGO_MANIFEST_DIR: /Users/kat/Projects/calico/examples/stm32f401re
    // CARGO_MANIFEST_PATH: /Users/kat/Projects/calico/examples/stm32f401re/Cargo.toml
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "".to_string());
    if out_dir == "" {
        panic!("failed to determine OUT_DIR")
    }

    // Debug logging for test generation.
    println!("Found test: {}", test_name);

    let mut test_desc: String = String::new();

    // If a manual description is specified as a
    // test attribute, then use that description.
    //
    // Otherwise, extra any doc attributes from the
    // function that may be present to use the doc
    // comments as the description.
    #[cfg(feature = "metadata")]
    if let Some(desc) = args.description {
        test_desc = desc;
    }

    // Attempt to extra a doc string as a test description.
    if test_desc.is_empty() {
        for attr in &input_fn.attrs {
            if let Some(doc) = super::extract_doc_string(attr) {
                test_desc.push_str(&doc);
                test_desc.push('\n'); // Preserve line breaks if multiple /// are used
            }
        }
    }

    println!("\t desc: {}", test_desc);

    // Inject the test start marker.
    input_fn.block.stmts.insert(
        0,
        syn::parse_quote! {
            calico::test::test_start();
        },
    );

    // Inject the test end marker.
    input_fn.block.stmts.insert(
        input_fn.block.stmts.len() - 1,
        syn::parse_quote! {
            calico::test::test_end();
        },
    );
    /*
    #[used]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".note.my_custom_note")]
        pub static MY_CUSTOM_NOTE: [u8; 8] = *b"rustnote"; */

    Ok(TokenStream::from(quote! {
        #input_fn
    }))
}
