// use minecraft_data::{prelude::MINECRAFT_DATA_DIR, FromMCDataVersionDir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ProtoDef = Namespace;

/**
 * [`ProtoDef Namespaces`] directly taken from protocol.json files,
 * subnamespaces are in the flattened `sub` HashMap
 *
 * [`ProtoDef Namespaces`]: https://github.com/ProtoDef-io/ProtoDef/blob/master/doc/protocol.md#protocol
 */
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct Namespace {
    #[serde(default)]
    pub types: HashMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub sub: HashMap<String, Namespace>,
}

// impl FromMCDataVersionDir for Namespace
// where
//     Self: Sized,
// {
//     fn from_version_paths(paths: &HashMap<String, String>) -> Option<Self> {
//         let mut path = std::path::PathBuf::from(paths.get("protocol").unwrap());
//         path.push("protocol.json");
//         Some(
//             serde_json::from_str(
//                 MINECRAFT_DATA_DIR
//                     .get_file(path)
//                     .unwrap()
//                     .contents_utf8()
//                     .unwrap(),
//             )
//             .unwrap(),
//         )
//     }
// }

// #[test]
// fn test() {
//     use minecraft_data::FromVersion;
//     for v in minecraft_data::supported_versions::SUPPORTED_VERSIONS {
//         println!("{:#?}", ProtoDef::from_version(v).unwrap());
//     }
// }
