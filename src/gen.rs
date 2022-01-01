use std::{
    collections::{HashMap, BTreeMap},
    path::{Path, PathBuf},
    rc::Rc,
};

use convert_case::{Case, Casing};
use indexmap::IndexMap;
use proc_macro2::{Ident, Span, TokenStream};
use serde_json::Value;

pub trait ClonableCasing: Casing + Clone {}

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

#[derive(Clone, Default)]
pub struct FieldName(pub Vec<Box<Rc<dyn convert_case::Casing>>>);

impl From<String> for FieldName {
    fn from(s: String) -> Self {
        Self(vec![Box::new(Rc::new(s))])
    }
}

impl FieldName {
    pub fn to_var_ident(&self) -> Ident {
        let ilvl = self.0.len() - 1;
        let last_elem = self.0.get(ilvl).unwrap();
        Ident::new(
            &format!("{}{}", "_".repeat(ilvl), last_elem.to_case(Case::Snake)),
            Span::call_site(),
        )
    }
    pub fn to_type_ident(&self) -> Ident {
        Ident::new(
            &self
                .0
                .iter()
                .map(|a| a.to_case(Case::Pascal))
                .reduce(|mut a, b| {
                    a.push('_');
                    a.push_str(&b);
                    a
                })
                .unwrap(),
            Span::call_site(),
        )
    }
    pub fn get_ilvl(&self) -> usize {
        self.0.len()
    }
    pub fn to_struct_field(&self) -> String {
        self.0.last().unwrap().to_case(Case::Snake)
    }
    #[must_use]
    pub fn push(&self, elem: Box<Rc<dyn Casing>>) -> Self {
        let mut a = self.0.to_vec();
        a.push(elem);
        Self(a)
    }
}

pub type CodeGenFn = Box<dyn FnOnce(FieldName) -> TokenStream>;

/// the heart piece of code generation
/// contains all information about the ser, de and def functions
/// in form of the rusttype field rst
pub struct Type {
    pub ser_code_gen_fn: CodeGenFn,
    pub de_code_gen_fn: CodeGenFn,
    pub def_code_gen_fn: CodeGenFn,
    pub rst: RustType,
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Type").field("rst", &self.rst).finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RustType {
    Struct(BTreeMap<String, RustType>),
    Enum(Vec<RustType>),
    Option(Box<RustType>),
    Vec(Box<RustType>),
    Array(Box<RustType>, usize),
    Simple(Ident),
    None
}

pub fn resolve_pds_type_ancestors<'ret>(
    pds: &super::pds::ProtoDef,
    ctx_path: &'ret Path,
    type_name: &str,
) -> Option<&'ret Path> {
    // println!("{:?}", pds);
    for ancestor in ctx_path.ancestors() {
        // println!("{:?}", (ancestor, type_name));
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
            Some(f(
                TypeFunctionContext {
                    path,
                    natives,
                    pds,
                    typestore,
                },
                None,
            ))
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
