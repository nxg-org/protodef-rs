use std::collections::HashMap;

use serde_json::Value;

use crate::gen::{GetGenTypeResult, TypeFunctionContext};

pub fn native_switch(ctx: TypeFunctionContext, opts: Option<&Value>) -> GetGenTypeResult {
    let raw_options = opts.unwrap().to_owned();
    let options = raw_options.as_object().unwrap();
    let mut placeholder_args = <HashMap<String, String>>::default();
    let mut add_placeholder_arg = |old: &str, dollar_ref: &str| {
        placeholder_args.insert(dollar_ref[1..].to_string(), old.to_owned());
    };
    match options.get("compareTo") {
        Some(Value::String(prefixed)) if prefixed.starts_with('$') => {
            add_placeholder_arg("compareTo", prefixed);
        }
        Some(Value::String(ref_field_path)) => todo!(),
        _ => match options.get("compareToValue") {
            Some(Value::String(prefixed)) if prefixed.starts_with('$') => {
                add_placeholder_arg("compareToValue", prefixed);
            }
            Some(val) => todo!(),
            _ => panic!("neither provided compareTo nor compareToValue to switch call"),
        },
    };
    if !placeholder_args.is_empty() {
        return GetGenTypeResult::ReExport(Box::new(
            move |ctx: TypeFunctionContext, opts: Option<&Value>| -> GetGenTypeResult {
                let mut raw_options = raw_options.clone();
                let placeholder_args = placeholder_args.clone();
                for (alias, insertto) in placeholder_args.into_iter() {
                    raw_options
                        .as_object_mut()
                        .unwrap()
                        .insert(
                            insertto.to_owned(),
                            opts
                                .expect("did not provide options to switch alias")
                                .as_object()
                                .expect("provided something else than an object to switch alias")
                                .get(&alias)
                                .or_else(||panic!("did not provide alias \"{}\" for property \"{}\" for aliased switch call", alias, insertto))
                                .unwrap()
                                .to_owned()
                        );
                }
                native_switch(ctx, opts)
            },
        ));
    }
    todo!("actual switch implementation")
}
