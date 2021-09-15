use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::exercise::Exercise;
use crate::render::Render;

pub struct MathJax {}

impl Render for MathJax {
    fn generate_output_filename(&self, output_dir: &Path, input: &Path) -> PathBuf {
        let mut output = output_dir.to_path_buf();
        output.push(input.file_name().unwrap());
        output.set_extension("tw");

        output
    }

    fn begin_exercise(&self, exercise: &Exercise) -> String {
        let output = format!(
            "::StoryTitle

{}

:: StoryData
{{
    \"ifid\": \"{}\"
}}

:: UserScripts [script]

/* Import the mathjax library. */
importScripts([
    \"https://polyfill.io/v3/polyfill.min.js?features=es6\",
    \"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\"
]).then(() => {{
        $(document).on(':passageinit', () => window.location.reload(false) );
    }})
    .catch(err => console.error(`MathJax load error: ${{err}}`));

",
            exercise.title,
            exercise.uuid,
        );

        output
    }

    fn begin_passage(&self, id: &str) -> String {
        format!(":: {}\n\n", id)
    }

    fn text(&mut self, text: &str) -> String {
        text.to_string()
    }

    fn link(&self, text: &str, target: &str) -> String {
        format!("\n[[ {} -> {} ]]\n\n", text, target)
    }

    fn begin_choices(&self, text: &str) -> String {
        text.to_string()
    }

    fn begin_option(&self, _text: &str, _target: &str) -> String {
        String::new()
    }
}
