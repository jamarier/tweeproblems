// Structures to support the use of formulas in histories

//use lazy_static::lazy_static;
//use maplit::hashmap;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use yaml_rust::YamlLoader;

pub type Macros = HashMap<String, String>;

pub fn include_macros(macros: Macros, macros_files: &[PathBuf]) -> Macros {
    let mut macros = macros;

    for file in macros_files {
        let contents = fs::read_to_string(file).expect("Unable to read file");
        let docs = YamlLoader::load_from_str(&contents).unwrap();
        let doc = &docs[0];

        for (key, value) in doc.as_hash().unwrap() {
            macros.insert(
                key.as_str().unwrap().to_string(),
                value.as_str().unwrap().to_string(),
            );
        }
    }

    macros
}

/*
lazy_static! {
    pub static ref FORMULAS: Formulas = hashmap! {
        "ohm_law_.RI" => " A : I !  ohm : R !   I @ R @ *    V : ",
        "ohm_law_V.I" => " A : I !  V : V !  V @ I @ /    ohm : ",
        "ohm_law_VR." => " ohm : R !    V : V !     V @ R @ /   A : ",

        "parallel" => " ohm : R2 !  ohm : R1 ! R1 @ R2 @ * R1 @ R2 @ + / ohm : "
    };
}
*/
