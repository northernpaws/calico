use std::hash::{DefaultHasher, Hash, Hasher};

use calico_elf::TestDefinition;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::attributes::tests::validate::{TestFunc, ValidatedModule};

/// Takes in a parsed test function and it's module, and returns a token
/// stream that contains the test function, the ELF symbols for informing
/// the runner of the test, injection points for test running, etc.
pub(crate) fn test(test: &TestFunc, module: &ValidatedModule) -> TokenStream {
    let test_func = &test.func;

    let ident = &test.func.sig.ident;
    let ident_entrypoint = format_ident!("__{}_entrypoint", ident);

    // Generate a static symbol exported to the ELF that
    // describes the test and can be parsed by the runner.
    let sym = export_test_sym(test, ident_entrypoint, module.args.default_timeout);

    quote! {
        #test_func

        #sym
    }
}

/// Writes a out symbol to the binary that's placed in a special
/// section of the ELF that the test runner can read.
///
/// These symbols don't end up on the ROM of the target MCU, but
/// allow the ELF to be a one-stop binary that can be fed to the
/// runner both for flashing, and for extracting the embeded tests.
pub(crate) fn export_test_sym(
    test: &TestFunc,
    ident_entrypoint: Ident,
    default_timeout: Option<u32>,
) -> proc_macro2::TokenStream {
    // Get the #[cfg(...)] statements that where on the test function.
    //
    // These configure in/out functions depending on platform, project
    // features, etc. so we want to keep symbols associated with the
    // test function they where defined on in-sync.
    let cfgs = &test.cfgs;

    let should_panic = test.should_panic;
    let ignore = test.ignore;
    let test_name = &test.func.sig.ident;
    let timeout = test.timeout.or(default_timeout);

    // Generate a name used for the symbol variable.
    let ident_var = format_ident!("__{}_SYM", test_name.to_string().to_uppercase());

    // Encode the test definition to JSON so it can be
    // embedded as the symbol name in the ELF symbol tree.
    let sym_name = serde_json::to_string(&TestDefinition {
        disambiguator: _crate_local_disambiguator(),
        name: test_name.to_string(),
        ignored: ignore,
        should_panic,
        timeout,
    })
    .expect("failed to convert test definition to JSON");

    // Unfortunately the module path can not be extracted from the Span yet.
    // At least on stable rust. Tracking issue: https://github.com/rust-lang/rust/issues/54725
    // As a workaround we use `module_path!()` to get the module path at runtime.
    //#ident_entrypoint
    quote!(
        #(#cfgs)*
        #[used]
        //#[no_mangle]
        #[unsafe(link_section = ".calico.tests")]
        #[unsafe(export_name = #sym_name)]
        static #ident_var: (fn()->!,&'static str) = (||loop{}, module_path!());
    )
}

fn _hash(string: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    string.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn _crate_local_disambiguator() -> u64 {
    //copied from defmt ;)
    // We want a deterministic, but unique-per-macro-invocation identifier. For that we
    // hash the call site `Span`'s debug representation, which contains a counter that
    // should disambiguate macro invocations within a crate.
    _hash(&format!("{:?}", Span::call_site()))
}

fn _json_escape(string: &str) -> String {
    use std::fmt::Write;
    let mut escaped = String::new();
    for c in string.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '\"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            c if c.is_control() || c == '@' => write!(escaped, "\\u{:04x}", c as u32).unwrap(),
            c => escaped.push(c),
        }
    }
    escaped
}
