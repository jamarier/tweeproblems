use std::path::{Path, PathBuf};

use crate::exercise::Exercise;

pub trait Render {
    fn generate_output_filename(&self, output_dir: &Path, input_filename: &Path) -> PathBuf;

    fn begin_exercise(&self, exercise: &Exercise) -> String;
    fn end_exercise(&self, _exercise: &Exercise) -> String {
        String::new()
    }

    fn begin_passage(&self, id: &str) -> String;
    fn end_passage(&self, _id: &str) -> String {
        String::new()
    }


    fn text(&mut self, text: &str) -> String;
    fn link(&self, text: &str, target: &str) -> String;

    fn begin_choices(&self, text: &str) -> String;
    fn end_choices(&self, _text: &str) -> String {
        String::new()
    }

    fn begin_option(&self, text: &str, target: &str) -> String;
    fn end_option(&self, _id: &str) -> String {
        String::new()
    }

}
