use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::attributes::tests::validate::{InitFunc, TestFunc};

/// Generate code to invoke the specified function, passing
/// the provided Vec of arguments to the invokation.
fn invoke(func: &ItemFn, args: Vec<TokenStream>) -> TokenStream {
    let ident = &func.sig.ident;
    if func.sig.asyncness.is_some() {
        quote!(#ident(#(#args),*).await)
    } else {
        quote!(#ident(#(#args),*))
    }
}

/// Generate a code block ( in { ... }) to call the init function (if provided), call the test function and check the outcome.
pub(crate) fn call_test_fn(test_func: &TestFunc, init_func: Option<&InitFunc>) -> TokenStream {
    // Generate the tokens to invoke the init function, if one was defined.
    let init_expr = if let Some(init) = init_func {
        invoke(&init.func, vec![])
    } else {
        quote!(())
    };

    // If the test function signature accepts an input paramter,
    // then pass in state provided by the init function call.
    let run_call = if test_func.input.is_some() {
        invoke(&test_func.func, vec![quote!(state)])
    } else {
        invoke(&test_func.func, vec![])
    };

    quote!(
        {
            let outcome;
            {
                let state = #init_expr; // either init() or init().await or ()
                calico::harness::test::test_start();
                outcome = #run_call; // either test(state), test(state).await, test(), or test().await
                calico::harness::test::test_end();
            }
            calico::harness::test::check_outcome(outcome);
        }
    )
}
