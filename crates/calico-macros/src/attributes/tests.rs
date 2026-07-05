//! Proc macro attributes for flagging a module as a test suite.

use proc_macro::TokenStream;
use proc_macro_error2::abort;
use quote::{format_ident, quote};

use syn::{ItemMod, parse_macro_input};

use crate::attributes::tests::validate::ValidatedModule;

/// Handles generating code required by the tests.
mod codegen;
/// Parses macro input into tests.
mod parse;
/// Validates the tests parsed from a source project.
mod validate;

/// Takes in the parameters from the proc_macro_attribute and expands it to the result.
pub(crate) fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the arguments to the macro first.
    let macro_args = match parse::MacroArgs::parse(args) {
        Ok(args) => args,
        Err(e) => abort!(e),
    };

    // Parse the TokenStream into a syn AST node.
    let input_mod = parse_macro_input!(input as ItemMod);

    expand_inner(macro_args, input_mod)
}

pub(crate) fn expand_inner(args: parse::MacroArgs, input_mod: ItemMod) -> TokenStream {
    // Extract the function name for use as the test name/identifier.
    let mod_ident = &input_mod.ident;
    let mod_name_string = mod_ident.to_string();

    // Attributes about the test that we need
    // to try to determine or parse next.
    let mut test_name = mod_name_string;

    // Parse the attributes supplies as arguments to the macro.
    if let Some(name) = &args.name {
        test_name = name.clone();
    }

    println!("found suite: {}", test_name);

    let mut test_desc: String = String::new();

    // If a manual description is specified as a
    // test attribute, then use that description.
    //
    // Otherwise, extra any doc attributes from the
    // function that may be present to use the doc
    // comments as the description.
    #[cfg(feature = "metadata")]
    if let Some(desc) = &args.description {
        test_desc = desc.clone();
    }

    // Attempt to extra a doc string as a test description.
    if test_desc.is_empty() {
        for attr in &input_mod.attrs {
            if let Some(doc) = super::extract_doc_string(attr) {
                test_desc.push_str(&doc);
                test_desc.push('\n'); // Preserve line breaks if multiple /// are used
            }
        }
    }

    println!("\t desc: {}", test_desc);

    // Attempt to parse a test module from the supplied module input.
    let module = parse::Module::from(input_mod);

    for test in &module.functions {
        println!("\ttest: {}", test.name);
    }

    // Validate the parsed module and tests.
    let validated_module = ValidatedModule::validate_module_and_args(module, args);

    // TODO: validation

    // Extract the tokens we didn't parse so they're written back to the source module.
    let untouched_tokens = &validated_module.untouched_tokens;

    // Take the validate test functions, and write out the modified function code.
    //
    // The codegen adds test start/end markers, injects custom test breakpoints, and
    // adds ELF symbols used by the runner to extract test info directly from a binary.
    let tests = validated_module
        .tests
        .iter()
        .map(|test| codegen::test(test, &validated_module));

    // Take any init functions defined in the test module and emit those as well.
    let init_fns = validated_module.init_funcs.values().map(|i| &i.func);

    // Format the module name as an identifier so it can be injected into the macro output.
    let mod_name = format_ident!("{}", validated_module.module_name);

    // Write out the modified module.
    //
    // This modifies the defined test functions to inject
    // linker sections with test metadata, test marker fn
    // calls, etc. that are used by the test runner.
    quote!(
        mod #mod_name {
            #(#untouched_tokens)*

            #(#init_fns)*

            #(#tests)*
        }
    )
    .into()
}
