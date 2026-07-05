use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro::TokenStream;
use proc_macro_error2::abort;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Attribute, Item, ItemFn, ItemMod, parse_macro_input};

/// Defines the arguments that can be supplied
/// to the `tests` macro when invoked.
#[derive(Debug, FromMeta)]
pub(crate) struct MacroArgs {
    /// Optionally set the name of the test suite.
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

    /// Disables resetting the target between test invokations.
    ///
    /// This is useful for tests where you want to run a series
    /// of seperate tests on the same target state.
    pub disable_reset: Option<bool>,

    /// Sets the default timeout for all tests.
    pub default_timeout: Option<u32>,
}

impl MacroArgs {
    /// Parse a set of macro arguments from the supplied arguments token stream.
    pub(crate) fn parse(args: TokenStream) -> Result<Self, syn::Error> {
        let attr_args = NestedMeta::parse_meta_list(args.into())?;
        let macro_args = MacroArgs::from_list(&attr_args)?;
        Ok(macro_args)
    }
}

/// Defines the attributes available on a #[test] in a tests module.
#[derive(Debug, FromMeta, Default)]
pub(crate) struct TestAttributes {
    /// Optionally set the name of the test.
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

    /// Specifies the test init function to use.
    #[darling(default)]
    pub init: Option<syn::Ident>,
}

impl TestAttributes {
    pub fn from_attr(attr: &Attribute) -> Self {
        match &attr.meta {
            syn::Meta::Path(_) => TestAttributes::default(),
            meta => match TestAttributes::from_meta(meta) {
                Ok(test_attr) => test_attr,
                Err(e) => abort!(
                    attr,
                    "failed to parse `test` attribute. Must be of the form #[test(init = init_function)]: {}",
                    e
                ),
            },
        }
    }
}

pub(crate) struct TimeoutAttribute {
    pub value: u32,
}

impl syn::parse::Parse for TimeoutAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let value_lit: syn::LitInt = input.parse()?;
        let value = value_lit.base10_parse::<u32>()?;

        Ok(TimeoutAttribute { value })
    }
}

impl TimeoutAttribute {
    fn from_attr(attr: &Attribute) -> Self {
        match attr.parse_args::<TimeoutAttribute>() {
            Ok(timeout_attr) => timeout_attr,
            Err(e) => {
                abort!(
                    attr,
                    "failed to parse `timeout` attribute. Must be of the form #[timeout(10)] where 10 is the timeout in seconds. Error: {}",
                    e
                );
            }
        }
    }
}

/// Represents an attribute attached to a function inside a test module.
pub(crate) enum ModuleFnAttribute {
    /// Marks a function in a module as the initialization
    /// method to run before every test.
    Init,
    /// Marks a function in a module as a test.
    Test(TestAttributes),
    /// Indicates that a test should panic when running.
    ShouldPanic,
    /// Indicates that a test should be ignored.
    Ignore,
    /// Sets a timeout for the test.
    Timeout(TimeoutAttribute),
}

impl ModuleFnAttribute {
    /// Attempts to convert a `syn::Attribute` into a `FuncAttribute`.
    fn try_from_attr(attr: &Attribute) -> Option<Self> {
        let ident = attr.path().get_ident()?.to_string();
        Some(match ident.as_str() {
            "init" => ModuleFnAttribute::Init,
            "test" => ModuleFnAttribute::Test(TestAttributes::from_attr(attr)),
            "should_panic" => ModuleFnAttribute::ShouldPanic,
            "ignore" => ModuleFnAttribute::Ignore,
            "timeout" => ModuleFnAttribute::Timeout(TimeoutAttribute::from_attr(attr)),
            _ => return None,
        })
    }
}

/// Represent a function discovered inside a test module.
pub(crate) struct ModuleFn {
    /// The original parsed function that this ModuleFn was generated from.
    pub func: ItemFn,
    pub name: String,
    /// Attributes attached to this function.
    pub attributes: Vec<(ModuleFnAttribute, proc_macro2::Span)>,
}

impl From<ItemFn> for ModuleFn {
    /// Allows for parsing a syn::ItemFn into a ModuleFn.
    fn from(mut func: ItemFn) -> Self {
        let mut attributes = vec![];
        func.attrs.retain(|attr| {
            if let Some(func_attr) = ModuleFnAttribute::try_from_attr(attr) {
                attributes.push((func_attr, attr.path().span()));
                false
            } else {
                true
            }
        });

        Self {
            name: func.sig.ident.to_string(),
            func,
            attributes,
        }
    }
}

/// Encapsultes a parsed test module.
pub(crate) struct Module {
    /// The name of the module.
    pub name: String,
    /// Test functions discovered in the module, and their attributes.
    pub functions: Vec<ModuleFn>,
    /// Module tokens we weren't sure how to handle in the first pass.
    pub untouched_tokens: Vec<Item>,
}

/// Parse an ItemMod into a test module.
impl From<ItemMod> for Module {
    fn from(module: ItemMod) -> Self {
        // Extract the items in the module.
        //
        // A module with no items can contain no tests,
        // so we need to error on that condition.
        let Some((_, items)) = module.content else {
            abort!(module, "module must have functions",);
        };

        let mut untouched_tokens = vec![];
        let mut functions = vec![];
        for item in items {
            match item {
                // For any functions found in the module, attempt to parse them
                // as a test module function and it's associated attributes.
                Item::Fn(f) => functions.push(ModuleFn::from(f)),
                _ => untouched_tokens.push(item),
            }
        }

        Self {
            name: module.ident.to_string(),
            functions,
            untouched_tokens,
        }
    }
}
