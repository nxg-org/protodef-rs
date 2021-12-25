use std::{collections::HashMap, path::PathBuf};

use indexmap::IndexMap;

pub type TypeName = String;
pub type FieldName = String;
pub type SerializationCode = proc_macro2::TokenStream;
pub type DeserializationCode = proc_macro2::TokenStream;
pub type TypeDefinitionCode = proc_macro2::TokenStream;

pub type CodeGenFn = Box<
    dyn Fn(TypeName, FieldName) -> (TypeDefinitionCode, SerializationCode, DeserializationCode),
>;

struct Cx<'a, Tree> {
    tree: &'a mut Tree,
    path: PathBuf,
}

pub type TypeGenFnOptions = Option<serde_json::Value>;

pub struct TypeGenFn<'ret>(
    Box<
        dyn Fn() -> Box<
                dyn FnMut(
                    TypeGenContext<'_>,
                    Cx<'_, StructBuilder>,
                    TypeGenFnOptions,
                ) -> Option<TypeGenFn<'ret>>,
            > + Sync,
    >,
);

// impl TypeGenFn<'_> {
//     fn box_clone<'a>(&self) -> Box<dyn Fn(TypeGenContext<'_>, Cx<'_, StructBuilder>, TypeGenFnOptions) -> Option<TypeGenFn<'a>> + Sync> {

//     }
// }

pub struct TypeGenContext<'a> {
    pub pds: &'a mut super::pds::ProtoDef,
    pub nstree_path: PathBuf,
    pub types: &'a mut HashMap<PathBuf, CodeGenFn>,
}
pub type StructBuilder = IndexMap<PathBuf, CodeGenFn>;

pub type TypePath = PathBuf;

pub type TypeStore = HashMap<TypePath, CodeGenFn>;

// type PdsContext<'a> = Cx<'a, super::pds::ProtoDef>;

pub enum RustType {
    Struct(IndexMap<String, RustType>),
    Enum(Vec<RustType>),
    Option(Box<RustType>),
    Single(proc_macro2::Ident),
}

pub type Fields = IndexMap<String, proc_macro2::Ident>;

pub type Natives<'a> = HashMap<&'a str, TypeGenFn<'a>>;

lazy_static::lazy_static!(
    pub static ref BUILTIN_NATIVES: Natives<'static> = {
        let mut ret = HashMap::default();

        macro_rules! get_natives {
            ($($native_name:expr => $native_fun_name:ident $buf_getter:ident $buf_putter:ident $rstype:expr)+) => {
                $(
                    fn $native_fun_name<'ret>(
                        _: TypeGenContext<'_>,
                        structbuilder_cx: Cx<'_, StructBuilder>,
                        _: Option<serde_json::Value>,
                    ) -> Option<TypeGenFn<'ret>> {
                        structbuilder_cx.tree.insert(
                            structbuilder_cx.path,
                            Box::new(|type_name, field_name| {
                                (
                                    quote::quote! {
                                        let #field_name: $rstype = buf.$buf_getter();
                                    },
                                    quote::quote! {
                                        buf.$buf_putter(#field_name);
                                    },
                                    quote::quote! {
                                        pub type #type_name = $rstype;
                                    }
                                )
                            })
                        );
                        None
                    }
                    ret.insert($native_name, TypeGenFn(Box::new(||Box::new(&$native_fun_name))));
                )*
            };
        }

        get_natives!(
            "varlong" => native_varlong get_var_long put_var_long "i64"
            "varint" => native_varint get_var_int put_var_int "i32"
            "u8"  => native_u8 get_u8 put_u8 "u8"
            "i8"  => native_i8 get_i8 put_i8 "i8"
            "u16" => native_u16 get_u16 put_u16 "u16"
            "i16" => native_i16 get_i16 put_i16 "i16"
            "u32" => native_u32 get_u32 put_u32 "u32"
            "i32" => native_i32 get_i32 put_i32 "i32"
            "u64" => native_u64 get_u64 put_u64 "u64"
            "i64" => native_i64 get_i64 put_i64 "i64"
            "u128" => native_u128 get_u128 put_u128 "u128"
            "i128" => native_i128 get_i128 put_i128 "i128"
            "f32" => native_f32 get_f32 put_f32 "f32"
            "f64" => native_f64 get_f64 put_f64 "f64"
            "boolean" => native_bool get_bool put_bool "bool"
        );
        ret.insert("switch", TypeGenFn(Box::new(||Box::new(native_switch))));

        ret
    };
);

fn native_switch<'ret>(
    _typegencx: TypeGenContext<'_>,
    _structbuilder_cx: Cx<'_, StructBuilder>,
    opt: Option<serde_json::Value>,
) -> Option<TypeGenFn<'ret>> {
    let raw_options = opt.unwrap();
    let options = raw_options.as_object().unwrap();
    let mut placeholder_arguments: HashMap<String, String> = Default::default();
    let mut add_placeholder_arg = |old: &str, dollar_ref: &String| {
        placeholder_arguments.insert(dollar_ref[1..].to_string(), old.to_owned());
    };
    let mut enum_code = proc_macro2::TokenStream::default();
    match options.get("compareTo") {
        Some(serde_json::Value::String(s)) if s.starts_with("$") => {
            add_placeholder_arg("compareTo", s)
        }
        Some(serde_json::Value::String(s)) => {
            let _field_name = s;
            let fields = options.get("fields").unwrap().as_object().unwrap();
            enum_code.extend(quote::quote! {
                match r#type
            });
            for (_field_matcher, _field_type) in fields {
                // let (field_type, field_path) = todo!();
                // // lookup_type(field_type, ...);
                // enum_code.extend(quote::quote! { #field_matcher => { } });
            }
        }
        _ => match options.get("compareToValue") {
            Some(serde_json::Value::String(s)) if s.starts_with("$") => {
                add_placeholder_arg("compareToValue", s)
            }
            Some(serde_json::Value::String(ident)) => {}
            _ => panic!(),
        },
    }
    if placeholder_arguments.len() > 0 {
        Some(TypeGenFn(Box::new(move || {
            let mut raw_options = raw_options.clone();
            let placeholder_arguments = placeholder_arguments.clone();
            Box::new(
                move |typegencx: TypeGenContext<'_>,
                      structbuilder_cx: Cx<'_, StructBuilder>,
                      opts: Option<serde_json::Value>| {
                    let placeholder_options = opts.unwrap();
                    for (alias, insertto) in &placeholder_arguments {
                        if let Some(obj) = raw_options.as_object_mut() {
                            obj.insert(
                                insertto.clone(),
                                placeholder_options.get(&alias.to_owned()).unwrap().clone(),
                            );
                        }
                    }
                    native_switch(typegencx, structbuilder_cx, Some(raw_options.to_owned()))
                },
            )
        })))
        // Some(TypeGenFn(Box::new(
        //     move |typegencx: TypeGenContext<'_>,
        //           structbuilder_cx: Cx<'_, StructBuilder>,
        //           opts: Option<serde_json::Value>| {
        //         let placeholder_options = opts.unwrap();
        //         for (alias, insertto) in &placeholder_arguments {
        //             if let Some(obj) = raw_options.as_object_mut() {
        //                 obj.insert(
        //                     insertto.clone(),
        //                     placeholder_options.get(&alias.to_owned()).unwrap().clone(),
        //                 );
        //             }
        //         }
        //         native_switch(typegencx, structbuilder_cx, Some(raw_options.to_owned()))
        //     },
        // )))
    } else {
        None
    }
}

pub fn get_hm_of_code_gen_fn<'a>(
    pds: &'a mut super::pds::ProtoDef,
    natives: &'static Natives<'a>,
) -> HashMap<PathBuf, CodeGenFn> {
    let ret = Default::default();

    for (path, pds_type) in pds.typemap.to_owned() {
        match pds_type {
            super::pds::Type::Reference(string) => {}
            super::pds::Type::Container(fields) => {}
            super::pds::Type::Call(mut call_type, options) => {
                let cg_fn = type_with_cx_and_natives_to_type_gen_fn(
                    Cx {
                        tree: pds,
                        path: path.to_owned(),
                    },
                    call_type.as_mut().to_owned(),
                    natives,
                );
                let mut types = Default::default();
                let tg_cx = TypeGenContext {
                    pds,
                    nstree_path: path.to_owned(),
                    types: &mut types,
                };
                let mut sb = StructBuilder::default();
                cg_fn.0()(
                    tg_cx,
                    Cx {
                        tree: &mut sb,
                        path: "/".into(),
                    },
                    Some(options),
                );
            }
        }
    }
    ret
}

fn type_with_cx_and_natives_to_type_gen_fn<'a>(
    cx: Cx<'_, super::pds::ProtoDef>,
    r#type: super::pds::Type,
    natives: &'static Natives<'a>,
) -> TypeGenFn<'a> {
    match r#type {
        super::pds::Type::Reference(native) if native == "native" => {
            let native_name = cx.path.file_name().unwrap().to_str().unwrap().to_string();
            let func = &natives
                .get(&native_name[..])
                .expect(&format!("couldn't resolve native type '{:#}'", native_name))
                .clone()
                .0;
            TypeGenFn(Box::new(move || func()))
        }
        super::pds::Type::Reference(type_name) => {
            let (t, t_path) = lookup_ref(&cx, &type_name).expect(&format!(
                "couldn't resolve reference '{}' at namespace path {:?}",
                type_name, &cx.path
            ));
            return type_with_cx_and_natives_to_type_gen_fn(
                Cx {
                    tree: cx.tree,
                    path: t_path,
                },
                t,
                natives,
            );
        }
        super::pds::Type::Call(mut call_type, options) => {
            // let tgen_fn = type_with_cx_and_natives_to_type_gen_fn(cx, *call_type, natives);
            // tgen_fn.0(cx, )
            let cg_fn = type_with_cx_and_natives_to_type_gen_fn(
                Cx {
                    tree: cx.tree,
                    path: cx.path.clone(),
                },
                call_type.as_mut().to_owned(),
                natives,
            );
            let mut types = Default::default();
            let tg_cx = TypeGenContext {
                pds: cx.tree,
                nstree_path: cx.path,
                types: &mut types,
            };
            let mut sb = StructBuilder::default();
            cg_fn.0()(
                tg_cx,
                Cx {
                    tree: &mut sb,
                    path: "/".into(),
                },
                Some(options),
            )
            .expect("called type did not return a type definition which takes options.")
        }
        super::pds::Type::Container(_) => {
            panic!(
                "container output doesn't take any arguments. namespace path '{:?}'",
                &cx.path
            );
        }
    }
}

fn lookup_ref(
    cx: &Cx<super::pds::ProtoDef>,
    t_name: &String,
) -> Option<(super::pds::Type, PathBuf)> {
    for ancestor in cx.path.ancestors() {
        let mut type_path = ancestor.to_path_buf();
        type_path.push(&t_name);
        if let Some(t) = cx.tree.typemap.get(&type_path) {
            return Some((t.to_owned(), type_path));
        }
    }
    None
}
