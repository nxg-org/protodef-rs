use std::rc::Rc;

use serde_json::Value;

use crate::{
    gen::{FieldName, GetGenTypeResult, Type, TypeFunctionContext},
    merge::merge,
    pds::val_to_type,
};

pub fn native_container(
    TypeFunctionContext {
        pds,
        path,
        typestore,
        natives,
    }: TypeFunctionContext,
    opts: Option<&Value>,
) -> GetGenTypeResult {
    if let Some(opts) = opts {
        let json_fields = opts.as_array().unwrap();
        let mut fields_vec: Vec<(Option<FieldName>, Type)> =
            Vec::new();
        for field in json_fields {
            let field = field.as_object().unwrap();
            let name = match field.get("name") {
                Some(Value::String(name)) => Some(name.to_owned()),
                None if matches!(field.get("anon"), Some(Value::Bool(true))) => None,
                _ => panic!("container fields have to either have a name or be anonymous"),
            };
            let t = val_to_type(
                field.get("type").unwrap(),
                Box::new(|| {
                    panic!("you cannot use \"native\" in here, stupid");
                }),
            );
            fields_vec.push((
                // this doesn't work without doing a manual map for **some** reason
                // (reason being 'static lifetime)
                #[allow(clippy::manual_map)]
                match name {
                    Some(name) => Some(Box::new(Rc::new(name))),
                    None => None,
                },
                match crate::gen::get_gen_type(
                    TypeFunctionContext {
                        pds,
                        path: path.clone(),
                        typestore,
                        natives,
                    },
                    &t,
                ) {
                    Some(GetGenTypeResult::Done(t)) => t,
                    _ => panic!("provided aliased or incomplete type in container field"),
                },
            ));
        }
        GetGenTypeResult::Done(merge(fields_vec))
    } else {
        GetGenTypeResult::ReExport(Box::new(&native_container))
    }
}
