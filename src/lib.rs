pub mod abi;
use std::{collections::HashMap, rc::Rc};

use abi::ImportFunc;
use anyhow::{bail, Context, Result};
use wasmparser::{Import, Parser, Payload, RecGroup, TypeRef};

pub fn parse_import_funcs(wasm_binary: &[u8]) -> Result<Vec<ImportFunc>> {
    struct UntypedImportFunc<'a> {
        /// The module being imported from.
        pub module: &'a str,
        /// The name of the imported item.
        pub name: &'a str,
        /// The type index of the imported item.
        pub type_ref: u32,
    }
    impl<'a> TryFrom<Import<'a>> for UntypedImportFunc<'a> {
        type Error = anyhow::Error;

        fn try_from(import: Import<'a>) -> Result<Self> {
            if let TypeRef::Func(index) = import.ty {
                Ok(UntypedImportFunc {
                    module: import.module,
                    name: import.name,
                    type_ref: index,
                })
            } else {
                bail!("Import type is not a function");
            }
        }
    }

    let mut import_funcs: Vec<UntypedImportFunc> = Vec::new();
    let mut types: HashMap<usize, Rc<RecGroup>> = HashMap::new();

    let parser = Parser::new(0);
    for payload in parser.parse_all(&wasm_binary) {
        match payload? {
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import.context("Failed to parse import entry")?;
                    if matches!(import.ty, TypeRef::Func(..)) {
                        import_funcs.push(import.try_into().unwrap());
                    } else {
                        bail!("ImportSection other than Func is not supported");
                    }
                }
            }
            Payload::TypeSection(reader) => {
                for (offset, record_group) in reader.into_iter().enumerate() {
                    types.insert(
                        offset,
                        Rc::new(record_group.context("Failed to parse type")?),
                    );
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
    for func in import_funcs.iter() {
        if !func.is_c_abi() {
            bail!("Imported function type is not C ABI type");
        }
    }
    Ok(import_funcs)
}
