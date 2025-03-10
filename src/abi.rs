use std::{collections::HashSet, rc::Rc};

use wasmparser::{CompositeInnerType, FuncType, RecGroup, SubType};

/// An imported function in a WebAssembly module.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ImportFunc<'a> {
    /// The module being imported from.
    pub module: &'a str,
    /// The name of the imported item.
    pub name: &'a str,
    /// The type of the imported item.
    pub ty: Rc<RecGroup>,
}
impl ImportFunc<'_> {
    pub fn is_c_abi(&self) -> bool {
        let rg = &self.ty;
        !rg.is_explicit_rec_group()
            && rg
                .types()
                .next()
                .is_some_and(|sty| sty.is_final && is_c_abi_func(sty))
    }

    pub fn unwrap_func(&self) -> &FuncType {
        self.ty.types().next().unwrap().unwrap_func()
    }
}

fn is_c_abi_func(ty: &SubType) -> bool {
    match ty.composite_type.inner {
        CompositeInnerType::Func(ref func_ty) => is_c_abi(func_ty),
        _ => false,
    }
}
fn is_c_abi(func_ty: &FuncType) -> bool {
    func_ty.results().len() <= 1
}

pub fn forbidden_imports<'a, 'i>(
    imports: &'i [ImportFunc<'a>],
    blacklist: &'i [&str],
) -> Vec<&'i ImportFunc<'a>> {
    let blacklist: HashSet<&str> = blacklist.iter().cloned().collect();
    imports
        .iter()
        .filter(|import| blacklist.contains(import.name))
        .collect()
}
