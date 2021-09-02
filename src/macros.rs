// Structures to support the use of formulas in histories

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use yaml_rust::YamlLoader;

//--------------------------------------------
// Paths

pub fn locate_file(input: &Path, paths: &[String]) -> Result<PathBuf> {
    if input.is_absolute() {
        if !input.exists() {
            bail!("File {:?} doesn't exists.", input)
        } else {
            Ok(input.to_path_buf())
        }
    } else {
        for path in paths {
            let test_input = Path::new(path).join(input);
            //println!("testing file: {:?}", test_input);
            if test_input.exists() {
                return Ok(test_input);
            }
        }

        bail!("File {:?} doesn't exists.", input)
    }
}

//--------------------------------------------
// Macros

#[derive(Clone, Debug)]
pub struct Macros {
    pub macros: HashMap<String, String>,
    paths: Vec<String>,
}

impl Macros {
    pub fn new() -> Self {
        Macros {
            macros: HashMap::new(),
            paths: Vec::new(),
        }
    }

    pub fn add_paths(&mut self, new_paths: Vec<String>) {
        for path in new_paths {
            self.paths.push(path);
        }
    }

    pub fn include_macros(&mut self, macros_files: Vec<String>) {
        for file in macros_files {
            let file = locate_file(Path::new(&file), &self.paths)
                .unwrap_or_else(|_| panic!("file {:?} not found", file));
            let contents =
                fs::read_to_string(file).unwrap_or_else(|_| panic!("Unable to read file"));
            let docs = YamlLoader::load_from_str(&contents).unwrap();
            let doc = &docs[0];

            for (key, value) in doc.as_hash().unwrap() {
                self.macros.insert(
                    key.as_str().unwrap().to_string(),
                    value.as_str().unwrap().to_string(),
                );
            }
        }
    }
}
