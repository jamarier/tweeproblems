use anyhow::{bail, Result};
use clap::{App, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{write, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use uuid::Uuid;

mod magnitude;

use crate::magnitude::Magnitude;

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
    let mut lines = open_input_file(input_file)?;

    let title = locate_title(&mut lines)?;

    let mut output_lines = preface(&title);

    let mut _current_level = 0;
    let mut _current_passage = Vec::<String>::new();

    let mut variables: HashMap<String, Magnitude> = HashMap::new();

    for line in lines {
        let line = extract_variables(line?, &mut variables);
        output_lines.push(line);
    }

    println!("variables detectadas: {:?}", variables);

    write(output_file, output_lines.join("\n"))?;

    Ok(())
}

fn extract_variables(line: String, variables: &mut HashMap<String, Magnitude>) -> String {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"\(\(\s*([^=]+?)\s*=\s*(\d+\.?\d*(e\d+)?)\s*([^)]*)\s*\)\)").unwrap();
    }

    let mut out_vec = Vec::<String>::new();

    // search for inline code
    let mut it: usize = 0;
    for cap in RE.captures_iter(&line) {
        let m = cap.get(0).unwrap();
        out_vec.push(line[it..m.start()].to_owned());

        // variable name and value
        let var_name : String = (&cap[1]).to_string();
        let var_mag : Magnitude = Magnitude::new(cap[2].parse::<f32>().unwrap(), cap[4].to_owned());

        out_vec.push(format!("\\({}={}\\)", var_name, var_mag));

        // setting variable
        variables.insert(var_name, var_mag);

        it = m.end();
    }
    if it < line.len() {
        out_vec.push(line[it..].to_owned());
    }

    // no match. all in_line into output
    if out_vec.is_empty() {
        out_vec.push(line);
    }

    out_vec.join("")
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
