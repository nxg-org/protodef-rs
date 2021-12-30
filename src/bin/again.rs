use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use protodef_rs::gen::TypeStore;

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
    println!(
        "{:?}",
        protodef_rs::gen::get_gen_type(
            protodef_rs::gen::TypeFunctionContext {
                pds: &pds_pd,
                path: type_path.to_owned(),
                typestore: &mut ts,
                natives: &natives,
            },
            pds_pd
                .typemap
                .get(&(iter.next().unwrap().to_owned(), type_name.to_owned()))
                .unwrap()
        )
        .unwrap()
    );
}
