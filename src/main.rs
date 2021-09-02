use anyhow::{bail, Result};
use clap::{App, Arg};
use std::fs::write;
use std::path::{Path, PathBuf};

mod expression;
mod macros;
mod magnitude;
mod passage;

use crate::passage::Exercise;

fn main() -> Result<()> {
    let args = App::new("TwineProblems")
        .version("0.1")
        .author("Javier M Mora <jmmora@us.es>")
        .about("Take math/engineering exercises and generate twine stories to practice")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("paths")
                .help("Paths to look for sources and macros files")
                .short("p")
                .long("path")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let mut paths: Vec<String> = vec![];
    if let Some(p) = args.values_of("paths") {
        paths.extend(p.map(|x| x.to_string()));
    }
    paths.insert(0, String::from("."));

    let input_file = check_input_file(Path::new(args.value_of("INPUT").unwrap()))?;
    let input_file = macros::locate_file(input_file, &paths)?;
    let output_file = generate_output_filename(&input_file);

    println!("input file: {:?}", input_file);
    println!("output file: {:?}", output_file);

    let exercise = Exercise::load_exercise(&input_file, paths)?;

    let render = exercise.render();

    write(output_file, render)?;

    Ok(())
}

fn check_input_file(input: &Path) -> Result<&Path> {
    // Check if extension is correct
    match input.extension() {
        None => bail!("INPUT file without extension"),
        Some(ext) => {
            if ext != "yaml" {
                bail!("INPUT file extension isn't yaml.")
            }
        }
    }

    /*
        // Check if file exists
        if !input.exists() {
            bail!("INPUT file doesn't exists.")
        }
    */

    Ok(input)
}

fn generate_output_filename(input: &Path) -> PathBuf {
    let mut output = PathBuf::from(input);
    output.set_extension("tw");

    output
}
