use anyhow::{bail, Result};
use clap::{App, Arg};
use std::fs::{write, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use uuid::Uuid;

mod expression;
mod magnitude;
mod passage;

use crate::expression::DictVariables;
use crate::passage::Passage;

fn main() -> Result<()> {
    let args = App::new("TwineProblems")
        .version("0.1")
        .author("Javier M Mora <jmmora@us.es>")
        .about("Take problems and generate twine stories to practice")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input_file = check_input_file(Path::new(args.value_of("INPUT").unwrap()))?;
    let output_file = generate_output_filename(input_file);

    println!("input file: {:?}", input_file);
    println!("output file: {:?}", output_file);

    // Open the file in read-only mode (ignoring errors).
    let mut line_iterator = open_input_file(input_file)?;

    let title = locate_title(&mut line_iterator)?;

    let output_lines = preface(&title);

    let mut variables: DictVariables = DictVariables::new();

    let passage = Passage::read_passage(line_iterator, &mut variables);
    println!("passage: {:?}", passage);

    println!("variables detectadas: {:?}", variables);

    write(output_file, output_lines.join("\n"))?;

    Ok(())
}

fn check_input_file(input: &Path) -> Result<&Path> {
    // Check if extension is correct
    match input.extension() {
        None => bail!("INPUT file without extension"),
        Some(ext) => {
            if ext != "twp" {
                bail!("INPUT file extension isn't twp.")
            }
        }
    }

    // Check if file exists
    if !input.exists() {
        bail!("INPUT file doesn't exists.")
    }

    Ok(input)
}

fn generate_output_filename(input: &Path) -> PathBuf {
    let mut output = PathBuf::from(input);
    output.set_extension("tw");

    output
}

fn open_input_file(input: &Path) -> Result<std::io::Lines<BufReader<File>>> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);

    Ok(reader.lines())
}

fn locate_title(iterator: &mut std::io::Lines<BufReader<File>>) -> Result<String> {
    let mut title = String::new();

    while title.is_empty() {
        let line = match iterator.next() {
            None => bail!("Input file has not text/title"),
            Some(line) => line,
        }?;

        title = String::from(line.trim());
    }

    Ok(title)
}

fn preface(title: &str) -> Vec<String> {
    let mut output = Vec::new();

    output.push(format!(
        "::StoryTitle

{}

:: StoryData
{{
    \"ifid\": \"{}\"
}}

:: UserScripts [script]

/* Import the mathjax library. */
importScripts(\"https://polyfill.io/v3/polyfill.min.js?features=es6\");
importScripts(\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\");

:: Start

",
        title,
        Uuid::new_v4()
    ));

    output
}
