use std::{collections::HashMap, path::PathBuf};

use protodef_rs::gen::Type;

fn main() {
    // macro_rules! t {
    // ($($a:tt)*) => {
    // Type{
    // ser_code_gen_fn: Box::new(|field_name|quote::quote!{
    // got field name: #field_name
    // }),
    // de_code_gen_fn: Box::new(|field_name|quote::quote!{
    // got field name: #field_name
    // }),
    // def_code_gen_fn: Box::new(|field_name|quote::quote!{
    // got field name: #field_name
    // }),
    // rst: $($a)*,
    // }
    // }
    // }

    let pds = protodef_rs::pds::ProtoDef::default();
    let mut ts = protodef_rs::gen::TypeStore::default();
    let natives = protodef_rs::native::BUILTIN_NATIVES
        .iter()
        .map(|(a, b)| ((*a).to_owned(), b))
        .collect::<HashMap<String, _>>();
    macro_rules! ctx {
        () => {
            protodef_rs::gen::TypeFunctionContext {
                pds: &pds,
                path: PathBuf::from("/"),
                typestore: &mut ts,
                natives: &natives,
            }
        };
    }

    protodef_rs::native::BUILTIN_NATIVES.get("u64").unwrap()(ctx!(), None);
    let a = match protodef_rs::native::BUILTIN_NATIVES.get("u64").unwrap()(ctx!(), None) {
        protodef_rs::gen::GetGenTypeResult::Done(a) => a,
        _ => panic!("this shouldn't panic"),
    };
    let b = match protodef_rs::native::BUILTIN_NATIVES.get("varint").unwrap()(ctx!(), None) {
        protodef_rs::gen::GetGenTypeResult::Done(a) => a,
        _ => panic!("this shouldn't panic"),
    };

    let Type {
        ser_code_gen_fn,
        de_code_gen_fn,
        def_code_gen_fn,
        rst,
    } = protodef_rs::merge::merge_struct(vec![
        // (Some("a".to_owned()), t!(RustType::Simple("u64".to_owned()))),
        // (Some("b".to_owned()), t!(RustType::Simple("u64".to_owned()))),
        (Some("a".to_owned()), a),
        (Some("a".to_owned()), b),
    ]);

    println!("TYPE: {:#?}", rst);
    println!(
        "SER_CODE: {:#?}",
        ser_code_gen_fn("INSERT_IDENTIFIER_HERE".to_owned().into())
    );
    println!(
        "DE_CODE: {}",
        de_code_gen_fn("INSERT_IDENTIFIER_HERE".to_owned().into())
    );
    println!(
        "DEF_CODE: {}",
        def_code_gen_fn("INSERT_IDENTIFIER_HERE".to_owned().into())
    );
}