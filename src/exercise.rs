use anyhow::{bail, Result};
use std::fs;
use std::path::Path;
use yaml_rust::YamlLoader;

use crate::expression::DictVariables;
use crate::macros::Macros;
use crate::passage::{is_macros, PassageTree, PassageTitle};
use crate::render::Render;

#[derive(Clone)]
pub struct Exercise {
    pub title: String,
    pub passage_tree: PassageTree,
}

impl Exercise {
    pub fn load_exercise(file: &Path, paths: Vec<String>) -> Result<Exercise> {
        let contents = fs::read_to_string(file).expect("Unable to read file");
        let docs = YamlLoader::load_from_str(&contents).unwrap();
        let doc = &docs[0];

        let variables = DictVariables::new();

        let mut macros = Macros::new();
        macros.add_paths(vec![file.parent().unwrap().to_str().unwrap().to_string()]);
        macros.add_paths(paths);

        if let Some(paths) = is_macros("paths", doc) {
            macros.add_paths(paths);
        }
        if let Some(macros_files) = is_macros("macros", doc) {
            macros.include_macros(macros_files);
        }

        let title = doc["title"].as_str().unwrap().to_owned();

        let mut passage_trees = PassageTree::from_yaml(&doc["passages"], &variables, &macros);


        /*
        println!("\npassageTree: {:?}", passage_trees);
        for it in &passage_trees {
            println!("\npassageTree: {}", it);
        }
        */

        if passage_trees.len() != 1 {
            bail!("\nThe document in file {:?} doesn't start with an passage (it starts with alternative or concurrent group)", file);
        }

        Ok(Exercise {
            title,
            passage_tree: passage_trees.pop().unwrap(),
        })
    }

    pub fn render(&self, renderer: &mut dyn Render) -> String {
        let mut output = String::new();
        let passage_title = PassageTitle::new();

        output += &renderer.begin_exercise(self);

        output += &self.passage_tree.render(renderer, "", &passage_title );

        output += &renderer.end_exercise(self);
        output
    }
}
