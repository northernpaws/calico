use proc_macro_error2::abort;
use syn::{ItemFn, ReturnType, Type};

use crate::attributes::tests::parse::{ModuleFn, ModuleFnAttribute};

/// Represents a function defined in a test module that's
/// used for initializing one or more tests in the module.
pub(crate) struct InitFunc {
    /// Function name.
    pub name: String,
    /// The function symbols used for code generation.
    pub func: ItemFn,
    /// Return type of the init function, can be passed to tests.
    pub state: Option<Type>,
    /// Indicates if the init function is async.
    pub asyncness: bool,
}

impl From<ModuleFn> for InitFunc {
    fn from(func: ModuleFn) -> Self {
        let ModuleFn {
            func, attributes, ..
        } = func;
        for (attr, span) in attributes {
            match attr {
                ModuleFnAttribute::Init => {}
                ModuleFnAttribute::Test(_) => unreachable!(),
                _ => abort!(span, "The `#[init]` function can not have this attribute"),
            }
        }

        if check_fn_sig(&func.sig).is_err() || !func.sig.inputs.is_empty() {
            abort!(
                func.sig,
                "`#[init]` function must have signature `async fn() [-> Type]` (async/return type are optional)",
            );
        }

        if cfg!(not(feature = "embassy")) && func.sig.asyncness.is_some() {
            abort!(
                func.sig,
                "`#[init]` function can only be async if an async executor is enabled via feature",
            );
        }

        let state = match &func.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(.., ty) => Some(*ty.clone()),
        };
        InitFunc {
            name: func.sig.ident.to_string(),
            asyncness: func.sig.asyncness.is_some(),
            func,
            state,
        }
    }
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
