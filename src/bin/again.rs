use std::{collections::HashMap, path::PathBuf, rc::Rc};

use clap::Parser;
use protodef_rs::gen::{TypeStore, Type};

#[derive(Parser)]
struct App {
    #[clap(short)]
    pub file: PathBuf,
    pub nstree: String,
}

fn main() {
    let app = App::parse();
    let json_pd: protodef_rs::json::ProtoDef =
        serde_json::from_str(&std::fs::read_to_string(app.file).unwrap()).unwrap();
    let pds_pd: protodef_rs::pds::ProtoDef = json_pd.into();
    let mut ts = TypeStore::default();
    let natives = protodef_rs::native::BUILTIN_NATIVES
        .iter()
        .map(|(a, b)| ((*a).to_owned(), b))
        .collect::<HashMap<String, _>>();
    let type_path = PathBuf::from(app.nstree);
    let type_name = type_path.file_name().unwrap().to_str().unwrap();
    let mut iter = type_path.ancestors();
    iter.next();
    let Type {
        ser_code_gen_fn,
        de_code_gen_fn,
        def_code_gen_fn,
        rst,
    } = match protodef_rs::gen::get_gen_type(
        protodef_rs::gen::TypeFunctionContext {
            pds: &pds_pd,
            path: type_path.to_owned(),
            typestore: &mut ts,
            natives: &natives,
        },
        pds_pd
            .typemap
            .get(&(iter.next().unwrap().to_owned(), type_name.to_owned()))
            .unwrap(),
    )
    .unwrap() {
        protodef_rs::gen::GetGenTypeResult::Done(t) => t,
        protodef_rs::gen::GetGenTypeResult::ReExport(_) => panic!("got reexport when expected type"),
    };

    println!("TYPE: {:?}", rst);
    println!(
        "SER_CODE: {}",
        ser_code_gen_fn(Box::new(Rc::new("INSERT_IDENTIFIER_HERE".to_owned())), 0)
    );
    println!(
        "DE_CODE: {}",
        de_code_gen_fn(Box::new(Rc::new("INSERT_IDENTIFIER_HERE".to_owned())), 0)
    );
    println!(
        "DEF_CODE: {}",
        def_code_gen_fn(Box::new(Rc::new("INSERT_IDENTIFIER_HERE".to_owned())), 0)
    );
}
