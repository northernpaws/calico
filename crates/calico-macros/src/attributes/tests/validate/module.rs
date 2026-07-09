use std::collections::HashMap;

use proc_macro_error2::abort;
use quote::quote;
use syn::Item;

use crate::attributes::tests::{
    parse::{MacroArgs, Module, ModuleFn},
    validate::{AnnotatedFunction, InitFunc, OtherFunc, TestFunc},
};

/// Represents a test module the was parsed from a proc_macro,
/// and then validated to contain a valid test tree.
pub(crate) struct ValidatedModule {
    pub module_name: String,
    pub args: MacroArgs,
    pub default_init: Option<String>,
    /// Init functions that where parsed from the module.
    pub init_funcs: HashMap<String, InitFunc>,
    /// Tests in the module that where validated.
    pub tests: Vec<TestFunc>,
    /// Tokens that where parsed from the
    /// proc macro but not processed.
    ///
    /// These should be re-emitted to ensure
    ///
    pub untouched_tokens: Vec<Item>,
}

impl ValidatedModule {
    /// Validates the supplied parsed module from a proc macro, and returns the validated module.
    pub fn validate_module_and_args(module: Module, args: MacroArgs) -> Self {
        let Module {
            functions,
            mut untouched_tokens,
            ..
        } = module;

        let (default_init_func, tests, other_funcs) = categorize_functions(functions);
        let default_init = default_init_func.as_ref().map(|i| i.name.clone());

        let (init_funcs, other_funcs) =
            collect_init_functions(&tests, default_init_func, other_funcs);

        untouched_tokens.extend(other_funcs.into_iter().map(|f| Item::Fn(f.0.func))); // Add unused functions back to untouched tokens

        let m = ValidatedModule {
            module_name: module.name,
            init_funcs,
            tests,
            args,
            default_init: None,
            untouched_tokens: vec![],
        };

        m
    }

    pub(crate) fn init_function_for_test(&self, test: &TestFunc) -> Option<&InitFunc> {
        if let Some(custom_init) = test.custom_init.as_ref() {
            return Some(self.init_funcs.get(&custom_init.to_string()).unwrap());
        }

        self.default_init
            .as_ref()
            .map(|i| self.init_funcs.get(i.as_str()).unwrap())
    }
}

/// Loops through all the functions defined in a test
/// module and categorizes them based on their attributes.
fn categorize_functions(
    functions: Vec<ModuleFn>,
) -> (Option<InitFunc>, Vec<TestFunc>, Vec<OtherFunc>) {
    let mut init_func = None;
    let mut tests = vec![];
    let mut other_funcs = vec![];

    for func in functions {
        match AnnotatedFunction::from(func) {
            AnnotatedFunction::Init(i) if init_func.is_none() => init_func = Some(i),
            AnnotatedFunction::Init(i) => {
                abort!(
                    i.func.sig,
                    "only one `#[init]` function is allowed in a test module",
                );
            }
            AnnotatedFunction::Test(t) => tests.push(t),
            AnnotatedFunction::Other(f) => other_funcs.push(f),
        }
    }

    (init_func, tests, other_funcs)
}

/// Takes in a list of functions parsed from a test module
/// and categories them based on their attributes.
fn collect_init_functions(
    tests: &[TestFunc],
    default_init: Option<InitFunc>,
    mut other_funcs: Vec<OtherFunc>,
) -> (HashMap<String, InitFunc>, Vec<OtherFunc>) {
    let mut map = HashMap::new();

    if let Some(init_func) = default_init {
        map.insert(init_func.name.clone(), init_func);
    }

    for test in tests {
        if let Some(init_fn_ident) = test.custom_init.as_ref() {
            let init_fn_name = init_fn_ident.to_string();

            if map.contains_key(&init_fn_name) {
                continue;
            }

            if let Some(pos) = other_funcs
                .iter()
                .position(|f| f.0.func.sig.ident == init_fn_name)
            {
                let init_func = InitFunc::from(other_funcs.remove(pos).0);
                map.insert(init_fn_name, init_func);
            } else {
                abort!(
                    init_fn_ident,
                    "custom init function `{}` not found in the module",
                    init_fn_name
                );
            }
        }
    }
    (map, other_funcs)
}

fn validate_argument_type(test: &TestFunc, init_func: Option<&InitFunc>) {
    let init_func = init_func.map(|i| (i.name.as_str(), i.state.as_ref()));

    match (&test.input, init_func) {
        (Some(_), None) => {
            abort!(
                test.func.sig,
                "this test function has an argument but no `#[init]` function was provided",
            );
        }
        (Some(_), Some((init_fn_name, None))) => {
            abort!(
                test.func.sig,
                "this test function has an argument but the init function `{}` does not return a state",
                init_fn_name
            );
        }
        (Some(actual_type), Some((init_fn_name, Some(expected_type))))
            if actual_type != expected_type =>
        {
            abort!(
                actual_type,
                "this type must match the return type `{}` of the init function `{}`",
                type_ident(expected_type),
                init_fn_name
            );
        }
        _ => {}
    }
}

fn type_ident(ty: &syn::Type) -> String {
    let mut ident = String::new();
    let ty = format!("{}", quote!(#ty));
    ty.split_whitespace().for_each(|t| ident.push_str(t));
    ident
}
