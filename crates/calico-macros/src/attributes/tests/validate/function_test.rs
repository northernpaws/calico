use proc_macro_error2::abort;
use syn::{Attribute, Item, ItemFn, Type};

use crate::attributes::tests::parse::{ModuleFn, ModuleFnAttribute};

/// A validated test function in a module.
#[derive(Clone)]
pub(crate) struct TestFunc {
    /// The source function.
    ///
    /// Used by codegen to re-emit the function, with modifications to
    /// add test markers, start/end calls, and to inject test metadata.
    pub func: ItemFn,
    // The #[cfg(...)] statements that where on the test function.
    //
    // These configure in/out functions depending on platform, project
    // features, etc. so we want to keep symbols associated with the
    // test function they where defined on in-sync.
    pub cfgs: Vec<Attribute>,
    /// Inputs required to the test function.
    pub input: Option<Type>,
    /// Indicates if the function should panic.
    pub should_panic: bool,
    /// Indicates if the test should be ignored.
    pub ignore: bool,
    /// Indicates if the test is asyncronous.
    pub asyncness: bool,
    /// Specifies a timeout for the test.
    pub timeout: Option<u32>,
    /// Name of a custom init function for the test, if defined.
    pub custom_init: Option<syn::Ident>,
}

impl From<ModuleFn> for TestFunc {
    /// Constructs a validated test function from a parsed test function.
    fn from(func: ModuleFn) -> Self {
        let ModuleFn {
            func, attributes, ..
        } = func;

        let mut should_panic = false;
        let mut ignore = false;
        let mut timeout = None;
        let mut custom_init = None;
        for (attr, _span) in attributes {
            match attr {
                ModuleFnAttribute::Init => unreachable!(),
                ModuleFnAttribute::Test(attr) => custom_init = attr.init,
                ModuleFnAttribute::ShouldPanic => should_panic = true,
                ModuleFnAttribute::Ignore => ignore = true,
                ModuleFnAttribute::Timeout(t) => timeout = Some(t.value),
            }
        }

        if check_fn_sig(&func.sig).is_err() || func.sig.inputs.len() > 1 {
            abort!(
                func.sig,
                "`#[test]` function must have signature `async fn(state: Type)` (async/parameter are optional)",
            );
        }

        if cfg!(not(feature = "embassy")) && func.sig.asyncness.is_some() {
            abort!(
                func.sig,
                "`#[test]` function can only be async if an async executor is enabled via feature",
            );
        }

        let input = if func.sig.inputs.len() == 1 {
            Some(extract_single_value_arg(&func.sig.inputs[0]))
            // NOTE we cannot check the argument type matches `init.state` at this point
        } else {
            None
        };

        TestFunc {
            cfgs: extract_cfgs(&func.attrs),
            asyncness: func.sig.asyncness.is_some(),
            func,
            input,
            should_panic,
            ignore,
            timeout,
            custom_init,
        }
    }
}

fn extract_cfgs(attrs: &[Attribute]) -> Vec<Attribute> {
    let mut cfgs = vec![];

    for attr in attrs {
        if attr.path().is_ident("cfg") {
            cfgs.push(attr.clone());
        }
    }

    cfgs
}

// NOTE doesn't check the parameters or the return type
fn check_fn_sig(sig: &syn::Signature) -> Result<(), ()> {
    if sig.constness.is_none()
        && sig.unsafety.is_none()
        && sig.abi.is_none()
        && sig.generics.params.is_empty()
        && sig.generics.where_clause.is_none()
        && sig.variadic.is_none()
    {
        Ok(())
    } else {
        Err(())
    }
}

fn extract_single_value_arg(arg: &syn::FnArg) -> Type {
    if let syn::FnArg::Typed(pat) = arg {
        match &*pat.ty {
            syn::Type::Reference(_) => {}
            _ => return *pat.ty.clone(),
        }
    }
    abort!(arg, "parameter must be a single value, not a reference");
}
