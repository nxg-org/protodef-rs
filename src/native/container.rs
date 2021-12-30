use serde_json::Value;

use crate::{
    gen::{GetGenTypeResult, TypeFunctionContext},
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
    let json_fields = opts.unwrap().as_array().unwrap();
    let mut fields_vec = Vec::new();
    for field in json_fields {
        let field = field.as_object().unwrap();
        let name = match field.get("name") {
            Some(Value::String(name)) => Some(name),
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
            name,
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

    todo!()
}
