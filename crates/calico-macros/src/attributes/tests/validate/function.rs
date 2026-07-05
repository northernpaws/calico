use proc_macro_error2::abort;

use super::{InitFunc, OtherFunc, TestFunc};
use crate::attributes::tests::parse::{ModuleFn, ModuleFnAttribute};

/// Enum that represents one of the type of test functions
/// extracted from a calico::tests annotated module.
pub(crate) enum AnnotatedFunction {
    Init(InitFunc),
    Test(TestFunc),
    Other(OtherFunc),
}

impl From<ModuleFn> for AnnotatedFunction {
    fn from(func: ModuleFn) -> Self {
        enum FuncKind {
            Init,
            Test,
        }

        // Check the attributes on the function to determine it's type.
        let mut func_kind = None;
        for (attr, span) in &func.attributes {
            match attr {
                ModuleFnAttribute::Init if func_kind.is_none() => func_kind = Some(FuncKind::Init),
                ModuleFnAttribute::Test(_) if func_kind.is_none() => {
                    func_kind = Some(FuncKind::Test)
                }
                ModuleFnAttribute::Init | ModuleFnAttribute::Test(_) => {
                    abort!(
                        span,
                        "A function can only be marked with one of `#[init]` or `#[test]`"
                    );
                }
                _ => {}
            }
        }

        // Depending on the decoded function type,
        // parse the function as that type.
        match func_kind {
            Some(FuncKind::Init) => AnnotatedFunction::Init(InitFunc::from(func)),
            Some(FuncKind::Test) => AnnotatedFunction::Test(TestFunc::from(func)),
            None => AnnotatedFunction::Other(OtherFunc::from(func)),
        }
    }
}
