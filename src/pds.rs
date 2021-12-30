use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

// use super::json;

/**
 * pds::ProtoDef represents the root of a protodef
 * specification in a pds file format. Every Type
 * appearing will here be indexed by a path of
 * namespaces it is contained in
 */
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ProtoDef {
    pub typemap: HashMap<(PathBuf, TypeName), Type>,
}

pub type TypeName = String;

/**
 * pds::Type represents any kind of Type value in the
 * protocol, be it as a native declaration in the top
 * level namespace's types or a reference to those in
 * the definition of a packet or even a container or
 * switch on some other type.
 */
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Native(String),

    /**
     * The Reference kind would be represented in
     * a json protocol spec by a json string, that
     * to be resolved has to be looked up in the
     * parent namespaces. If it cannot be found then
     * the protocol spec file is written badly
     *
     * # Example
     *
     * ```rust
     * {
     *   "types": {
     *     "u64": "native" // <= a native reference, which will have to be treated
     *                     // differently by the compiler
     *   },
     *   "subnamespace": {
     *     "types": {
     *       "some_type": "u64" // <= the typical type of reference
     *     }
     *   }
     * }
     * ```
     *
     * resulting type tree:
     *
     * ```rust
     * {
     *   "/u64": "native",
     *   "/subnamespace/some_type": "u64"
     * }
     * ```
     */
    Reference(String),

    /**
     * The Container kind contains multiple fields
     * of types in a particular order. The order of
     * the fields is important because it is also the
     * order the fields have to be read from or
     * written to a buffer
     *
     * fields can also be anonymous, which merges them down in the container
     *
     * # Example
     *
     * ```rust
     * {
     *   "types": {
     *     "u64": "native",
     *     "u8": "native"
     *   },
     *   "subnamespace": {
     *     "types": {
     *       "a_packet": [
     *         "container",
     *         [
     *           {
     *             "name": "first_field",
     *             "type": "u64"
     *           },
     *           {
     *             "anon": true, // <= this specifies an anonymous field on the container,
     *                           // all fields it exports will be added to the parent container
     *             "type": [
     *               "container",
     *               [
     *                 {
     *                   "name": "same_level",
     *                   "type": "u8"
     *                 }
     *               ]
     *             ]
     *           }
     *         ]
     *       ]
     *     }
     *   }
     * }
     * ```
     *
     * resulting type tree:
     *
     * ```rust
     * {
     *   "/u64": "native",
     *   "/u8": "native",
     *   "/subnamespace/a_packet": {
     *     "/first_field": "u64",
     *     "/same_level": "u8"
     *   }
     * }
     * ```
     *
     */
    // Container(Vec<Field>),

    /**
     * The Call kind is similar to the Container in
     * syntax, having the type to call in the first
     * index and the "arguments" which to pass to
     * that call inside of the second index. it is
     * assumed you can only pass objects to the
     * second index.
     *
     * there are many builtin calls like switch,
     * option, array, buffer, ...
     * documented here:
     * <https://github.com/ProtoDef-io/ProtoDef/blob/master/doc/datatypes.md>
     *
     * # Example
     *
     * ```rust
     * {
     *   "types": {
     *     "u64": "native",
     *     "u8": "native",
     *     // any type referenced has to be declared as well, if for example
     *     // switch wasn't declared as native here, the compiler should also error.
     *     "switch": "native"
     *   },
     *   "subnamespace": {
     *     "types": {
     *       "a_packet": [
     *         "container",
     *         [
     *           {
     *             "name": "some_field",
     *             "type": "u8"
     *           },
     *           {
     *             "name": "special_field",
     *             "type": [
     *               "switch",
     *               {
     *                 "compareTo": "some_field",
     *                 "fields": {
     *                   "0": "u8",
     *                   "1": "u64"
     *                 },
     *                 "default": "void"
     *               }
     *             ]
     *           }
     *         ]
     *       ]
     *     }
     *   }
     * }
     * ```
     *
     * resulting type tree:
     *
     * ```rust
     * {
     *   "/u64": "native",
     *   "/u8": "native",
     *   "/switch": "native",
     *   "/subnamespace/a_packet": {
     *     "/some_field": "u8",
     *     "/special_field": [
     *       "u8",
     *       "u64",
     *       "void"
     *     ]
     *   }
     * }
     * ```
     */
    Call(Box<Type>, Value),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    pub name: Option<String>,
    pub r#type: Type,
}

impl From<super::json::ProtoDef> for ProtoDef {
    fn from(json_pds: super::json::ProtoDef) -> Self {
        let mut ret = Self::default();
        recursive_add_namespaces(&json_pds, PathBuf::from("/"), &mut ret);
        ret
    }
}

fn recursive_add_namespaces<P: AsRef<Path>>(
    json_ns: &super::json::ProtoDef,
    path: P,
    pds: &mut ProtoDef,
) {
    for (type_name, type_val) in &json_ns.types {
        pds.typemap.insert(
            (path.as_ref().to_owned(), type_name.to_owned()),
            val_to_type(type_val, || type_name.to_owned()),
        );
    }
    for (sub_name, sub_ns) in &json_ns.sub {
        let mut new_path = path.as_ref().to_owned();
        new_path.push(sub_name);
        recursive_add_namespaces(sub_ns, new_path, pds);
    }
}

pub fn val_to_type(val: &Value, native_fallback_name: impl Fn() -> String) -> Type {
    let native_error: Box<dyn Fn() -> String> = Box::new(|| {
        panic!("you cannot use \"native\" in here, stupid");
    });
    match val {
        Value::String(s) if s == "native" => Type::Native(native_fallback_name()),
        Value::String(s) => Type::Reference(s.to_owned()),
        Value::Array(arr) => {
            let mut arr_iter = arr.iter();
            let callee = arr_iter.next().unwrap();
            let opts = arr_iter.next().unwrap();
            Type::Call(Box::new(val_to_type(callee, native_error)), opts.to_owned())
        }
        _ => panic!(),
    }
}

// impl From<Value> for Type {
//     fn from(val: Value) -> Self {

//     }
// }

impl From<Value> for Field {
    fn from(v: Value) -> Self {
        let obj = v.as_object().unwrap();
        Self {
            name: match obj.get("name") {
                Some(Value::String(name)) => Some(name.to_owned()),
                _ => None,
            },
            r#type: val_to_type(obj.get("type").unwrap(), || {
                panic!("you cannot use \"native\" in here, stupid")
            }),
        }
    }
}

// impl minecraft_data::FromMCDataVersionDir for ProtoDef {
//     fn from_version_paths(paths: &HashMap<String, String>) -> Option<Self>
//     where
//         Self: Sized,
//     {
//         Some(
//             super::json::ProtoDef::from_version_paths(paths)
//                 .unwrap()
//                 .into(),
//         )
//     }
// }

// #[test]
// fn test() {
//     use minecraft_data::FromVersion;
//     for v in minecraft_data::supported_versions::SUPPORTED_VERSIONS {
//         println!("{:#?}", ProtoDef::from_version(v));
//     }
// }
