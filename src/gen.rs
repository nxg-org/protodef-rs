use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream;
use serde_json::Value;

#[derive(Default)]
pub struct TypeStore {
    pub types: HashMap<PathBuf, Type>,
}

pub struct TypeFunctionContext<'a> {
    pub pds: &'a super::pds::ProtoDef,
    pub path: PathBuf,
    pub typestore: &'a mut TypeStore,
    pub natives: &'a HashMap<String, &'static TypeFunction>,
}
pub type TypeFunction = Box<dyn Fn(TypeFunctionContext, Option<&Value>) -> GetGenTypeResult + Sync>;

pub type FieldName = String;
pub type CodeGenFn = Box<dyn FnOnce(FieldName) -> OutCode>;

pub struct OutCode {
    pub ser: TokenStream,
    pub de: TokenStream,
    pub def: TokenStream,
}

pub struct Type {
    pub code_gen_fn: CodeGenFn,
    pub rst: RustType,
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Type").field("rst", &self.rst).finish()
    }
}

#[derive(Debug)]
pub enum RustType {
    Struct(HashMap<String, RustType>),
    Enum(Vec<RustType>),
    Simple(String),
}

pub fn resolve_pds_type_ancestors<'ret>(
    pds: &super::pds::ProtoDef,
    ctx_path: &'ret Path,
    type_name: &str,
) -> Option<&'ret Path> {
    for ancestor in ctx_path.ancestors() {
        if pds
            .typemap
            .get(&(ancestor.to_owned(), type_name.to_owned()))
            .is_some()
        {
            return Some(ancestor);
        }
    }
    None
}

pub enum GetGenTypeResult {
    Done(Type),
    ReExport(TypeFunction),
}

impl std::fmt::Debug for GetGenTypeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Done(arg0) => f.debug_tuple("Done").field(arg0).finish(),
            Self::ReExport(_) => f.debug_tuple("ReExport").finish(),
        }
    }
}

pub fn get_gen_type(
    TypeFunctionContext {
        natives,
        pds,
        typestore,
        path,
    }: TypeFunctionContext<'_>,
    t: &super::pds::Type,
) -> Option<GetGenTypeResult> {
    use GetGenTypeResult::*;
    match t {
        crate::pds::Type::Native(native) => {
            let f = *natives
                .get(native)
                .or_else(|| panic!("no native type named \"{}\"", native))
                .unwrap();
            Some(f(TypeFunctionContext {
                path,
                natives,
                pds,
                typestore,
            }, None))
        }
        crate::pds::Type::Reference(new_name) => {
            let path = resolve_pds_type_ancestors(pds, &path, new_name)
                .unwrap()
                .to_owned();
            let t = pds
                .typemap
                .get(&(path.clone(), new_name.to_owned()))
                .unwrap();
            get_gen_type(
                TypeFunctionContext {
                    path,
                    natives,
                    pds,
                    typestore,
                },
                t,
            )
        }
        crate::pds::Type::Call(t, opts) => {
            match get_gen_type(
                TypeFunctionContext {
                    pds,
                    path: path.clone(),
                    typestore,
                    natives,
                },
                t,
            ) {
                Some(ReExport(f)) => Some(f(
                    TypeFunctionContext {
                        pds,
                        path,
                        typestore,
                        natives,
                    },
                    Some(opts),
                )),
                _ => panic!("given arguments to a type that doesn't take any"),
            }
        }
    }
}
