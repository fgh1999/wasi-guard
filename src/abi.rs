use alloc::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    vec::Vec,
};

use wasmparser::{
    CompositeInnerType, FuncType, Import, Parser, Payload, RecGroup, SubType, TypeRef,
};

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

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Failed to parse Wasm")]
    WasmParseError,
    #[error("Imported type is not a function")]
    InvalidImportType,
    #[error("ImportSection other than Func is not supported")]
    UnsupportedImportSectionError,
}

pub fn parse_import_funcs(wasm_binary: &[u8]) -> Result<Vec<ImportFunc>, ParseError> {
    struct UntypedImportFunc<'a> {
        /// The module being imported from.
        pub module: &'a str,
        /// The name of the imported item.
        pub name: &'a str,
        /// The type index of the imported item.
        pub type_ref: u32,
    }
    impl<'a> TryFrom<Import<'a>> for UntypedImportFunc<'a> {
        type Error = ParseError;
        fn try_from(import: Import<'a>) -> Result<Self, Self::Error> {
            if let TypeRef::Func(index) = import.ty {
                Ok(UntypedImportFunc {
                    module: import.module,
                    name: import.name,
                    type_ref: index,
                })
            } else {
                Err(ParseError::InvalidImportType)
            }
        }
    }

    let mut import_funcs: Vec<UntypedImportFunc> = Vec::new();
    let mut types: BTreeMap<usize, Rc<RecGroup>> = BTreeMap::new();

    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm_binary) {
        if payload.is_err() {
            return Err(ParseError::WasmParseError);
        }
        match payload.unwrap() {
            Payload::ImportSection(reader) => {
                for import in reader {
                    if import.is_err() {
                        return Err(ParseError::WasmParseError);
                    }
                    let import = import.unwrap();
                    if matches!(import.ty, TypeRef::Func(..)) {
                        import_funcs.push(import.try_into().unwrap());
                    } else {
                        return Err(ParseError::UnsupportedImportSectionError);
                    }
                }
            }
            Payload::TypeSection(reader) => {
                for (offset, record_group) in reader.into_iter().enumerate() {
                    if record_group.is_err() {
                        return Err(ParseError::WasmParseError);
                    }
                    types.insert(offset, Rc::new(record_group.unwrap()));
                }
            }
            _ => {}
        }
    }

    let import_funcs: Vec<ImportFunc> = import_funcs
        .into_iter()
        .map(|func| ImportFunc {
            module: func.module,
            name: func.name,
            ty: types.get(&(func.type_ref as usize)).unwrap().clone(),
        })
        .collect();

    // all imported functions' type should be C ABI type.
    // for func in import_funcs.iter() {
    //     if !func.is_c_abi() {
    //         bail!("Imported function type is not C ABI type");
    //     }
    // }
    Ok(import_funcs)
}

pub fn forbidden_imports<'a, 'i>(
    imports: &'i [ImportFunc<'a>],
    blacklist: &'i [&str],
) -> Vec<&'i ImportFunc<'a>> {
    let blacklist: BTreeSet<&str> = blacklist.iter().cloned().collect();
    imports
        .iter()
        .filter(|import| blacklist.contains(import.name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_import_funcs;

    #[test]
    fn parse_import_funcs_test() {
        use wasmparser::ValType;

        let wasm_binary = wat::parse_str(
r#"(module $rust-672937185f5392fa.wasm
    (type (;0;) (func))
    (type (;1;) (func (param i32)))
    (type (;2;) (func (param i32 i32)))
    (type (;3;) (func (param i32 i32) (result i32)))
    (type (;4;) (func (param i32) (result i32)))
    (type (;5;) (func (param i32 i32 i32) (result i32)))
    (type (;6;) (func (param i32 i32 i32)))
    (type (;7;) (func (param i32 i64 i32) (result i32)))
    (type (;8;) (func (param i32 i32 i32 i32) (result i32)))
    (type (;9;) (func (result i32)))
    (type (;10;) (func (param i32) (result i64)))
    (type (;11;) (func (param i32 i32 i32 i32)))
    (type (;12;) (func (param i32 i64 i32)))
    (type (;13;) (func (param i64 i64 i32) (result i64)))
    (type (;14;) (func (param i32 i64 i64)))
    (type (;15;) (func (param i32 i32 i32 i32 i32)))
    (type (;16;) (func (param i32 i32 i32 i32 i32) (result i32)))
    (type (;17;) (func (param i32 i32 i64 i32)))
    (type (;18;) (func (param i32 i32 i64)))
    (type (;19;) (func (param i32 i32 i32 i32 i32 i32 i32)))
    (type (;20;) (func (param i32 i32 i32 i32 i32 i32) (result i32)))
    (type (;21;) (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
    (type (;22;) (func (param i32 i64 i32 i32 i32 i32 i32 i32) (result i32)))
    (type (;23;) (func (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
    (type (;24;) (func (param i64 i32 i32) (result i32)))
    (type (;25;) (func (param i32 i64 i64 i64 i64)))
    (import "wasi_snapshot_preview1" "clock_time_get" (func $_ZN4wasi13lib_generated22wasi_snapshot_preview114clock_time_get17hf55eee9df40ce7deE (type 7)))
    (import "wasi_snapshot_preview1" "fd_write" (func $_ZN4wasi13lib_generated22wasi_snapshot_preview18fd_write17h12b230225e789f1eE (type 8)))
    (import "wasi_snapshot_preview1" "environ_get" (func $__imported_wasi_snapshot_preview1_environ_get (type 3)))
    (import "wasi_snapshot_preview1" "environ_sizes_get" (func $__imported_wasi_snapshot_preview1_environ_sizes_get (type 3)))
    (import "wasi_snapshot_preview1" "proc_exit" (func $__imported_wasi_snapshot_preview1_proc_exit (type 1)))
)"#).unwrap();
        let import_funcs = parse_import_funcs(&wasm_binary).unwrap();
        assert_eq!(import_funcs.len(), 5);
        assert_eq!(import_funcs[0].module, "wasi_snapshot_preview1");
        assert_eq!(import_funcs[0].name, "clock_time_get");
        assert!(import_funcs[0].is_c_abi());
        assert_eq!(import_funcs[0].unwrap_func().params().len(), 3);
        assert_eq!(import_funcs[0].unwrap_func().params()[1], ValType::I64);

        assert_eq!(import_funcs[4].name, "proc_exit");
        assert_eq!(import_funcs[4].unwrap_func().params().len(), 1);
        assert_eq!(import_funcs[4].unwrap_func().params()[0], ValType::I32);
    }
}
