use std::collections::HashMap;

use crate::gen::{RustType, Type};

pub fn merge(variants: Vec<(Option<String>, Type)>) -> Type {
    todo!("actually think of logic");

    let rst = if variants.len() == 1 && variants.iter().all(|(n, _)| n.is_none()) {
        let (_, t) = variants.iter().next().unwrap();
        t.rst
    } else if variants
        .iter()
        .all(|(n, t)| n.is_some() || matches!(t.rst, RustType::Struct(_)))
    {
        // merge down nested anonymous structs
        let hm = HashMap::new();
        for (name, t) in variants {
            match name {
                Some(name) => {hm.insert(name, t.rst);},
                None => if let RustType::Struct(hm) = t.rst {
                    for (sub_field_name, sub_field_type) in hm {
                        hm.insert(sub_field_name, sub_field_type);
                    }
                },
            }
        }
        RustType::Struct(hm)
    } else {
        todo!()
    };

    match rst {
        RustType::Struct(_) => todo!(),
        RustType::Enum(_) => todo!(),
        RustType::Simple(_) => todo!(),
    }

    variants.iter().for_each(|(name, t)| {});
    Type {
        code_gen_fn: todo!(),
        rst: todo!(),
    }
}
