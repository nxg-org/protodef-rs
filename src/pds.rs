use std::{collections::HashMap, path::PathBuf};

// use minecraft_data::FromVersion;

// use super::json;

/**
 * pds::ProtoDef represents the root of a protodef
 * specification in a pds file format. Every Type
 * appearing will here be indexed by a path of
 * namespaces it is contained in
 */
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ProtoDef{
    pub typemap: HashMap<PathBuf, Type>
}

/**
 * pds::Type represents any kind of Type value in the
 * protocol, be it as a native declaration in the top
 * level namespace's types or a reference to those in
 * the definition of a packet or even a container or
 * switch on some other type.
 */
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    /**
     * The Reference kind would be represented in
     * a json protocol spec by a json string, that
     * to be resolved has to be looked up in the
     * parent namespaces. If it cannot be found then
     * the protocol spec file is written badly
     *
     * spec example:
     *
     * ```json
     * {
     *   "types": {
     *     "u64": "native" /* <= a native reference,
     *            which will have to be treated
     *            differently by the compiler */
     *   },
     *   "subnamespace": {
     *     "types": {
     *       "some_type": "u64" // <= the typical type
     *                          // of reference
     *     }
     *   }
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
     * spec example:
     *
     * ```json
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
     *           }
     *         ]
     *       ]
     *     }
     *   }
     * }
     * ```
     */
    Container(Vec<Field>),

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
     * spec example:
     *
     * ```json
     * {
     *   "types": {
     *     "u64": "native",
     *     "u8": "native",
     *     /* any type referenced has
     *        to be declared as well, if
     *        switch wasn't declared as native
     *        here, the compiler should also error.
     *        the exact explanation of what switch is, here:
     *        <https://github.com/ProtoDef-io/ProtoDef/blob/master/doc/datatypes/conditional.md#>
     *      */
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
     */
    Call(Box<Type>, serde_json::Value),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    pub name: Option<String>,
    pub r#type: Type,
}

impl From<super::json::ProtoDef> for ProtoDef {
    fn from(mut json_pds: super::json::ProtoDef) -> Self {
        let mut ret = Self::default();
        recursive_add_namespaces(&mut json_pds, "/".into(), &mut ret);
        ret
    }
}

fn recursive_add_namespaces(json_ns: &super::json::ProtoDef, path: PathBuf, pds: &mut ProtoDef) {
    for (type_name, type_val) in &json_ns.types {
        let mut type_path = path.to_owned();
        type_path.push(type_name);
        pds.typemap.insert(type_path, type_val.to_owned().into());
    }
    for (sub_name, sub_ns) in &json_ns.sub {
        let mut new_path = path.to_owned();
        new_path.push(sub_name);
        recursive_add_namespaces(sub_ns, new_path, pds);
    }
}

impl From<serde_json::Value> for Type {
    fn from(val: serde_json::Value) -> Self {
        match val {
            serde_json::Value::String(s) => Type::Reference(s),
            serde_json::Value::Array(arr) => match (&arr[0], &arr[1]) {
                (serde_json::Value::String(s), serde_json::Value::Array(fields))
                    if s == "container" =>
                {
                    Self::Container(
                        fields
                            .to_owned()
                            .into_iter()
                            .map(|v| Field::from(v))
                            .collect(),
                    )
                }
                (v, val) => Type::Call(Box::new(Type::from(v.to_owned())), val.to_owned()),
            },
            _ => panic!(),
        }
    }
}

impl From<serde_json::Value> for Field {
    fn from(v: serde_json::Value) -> Self {
        let obj = v.as_object().unwrap();
        Self {
            name: match obj.get("name") {
                Some(serde_json::Value::String(name)) => Some(name.to_owned()),
                _ => None,
            },
            r#type: Type::from(obj.get("type").unwrap().to_owned()),
        }
    }
}

// impl minecraft_data::FromMCDataVersionDir for ProtoDef {
//     fn from_version_paths(paths: &HashMap<String, String>) -> Option<Self>
//     where
//         Self: Sized,
//     {
//         Some(json::ProtoDef::from_version_paths(paths).unwrap().into())
//     }
// }

// #[test]
// fn test() {
//     for v in minecraft_data::supported_versions::SUPPORTED_VERSIONS {
//         println!("{:#?}", ProtoDef::from_version(v));
//     }
// }
 