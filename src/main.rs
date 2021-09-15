use anyhow::{bail, Result};
use clap::{App, Arg};
use std::fs::write;
use std::path::Path;

mod exercise;
mod expression;
mod macros;
mod magnitude;
mod passage;

mod render;
mod render_mathjax;
mod render_reveal;

use crate::exercise::Exercise;
use crate::render::Render;
use crate::render_mathjax::MathJax;
use crate::render_reveal::Reveal;

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
            Arg::with_name("OUTPUTDIR")
                .help("Output directory")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("paths")
                .help("Paths to look for sources and macros files")
                .short("p")
                .long("path")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("render")
                .help("What render use")
                .short("r")
                .long("render")
                .possible_values(&["reveal"])
                .takes_value(true)
                .default_value("reveal"),
        )
        .get_matches();

    let mut paths: Vec<String> = vec![];
    if let Some(p) = args.values_of("paths") {
        paths.extend(p.map(|x| x.to_string()));
    }
    paths.insert(0, String::from("."));

    let mut reveal;
    let renderer: &mut dyn Render = match args.value_of("render").unwrap() {
        "reveal" => {
            reveal = Reveal::new();
            &mut reveal
        }
        _ => panic!("unknown"),
    };

    let input_file = check_input_file(Path::new(args.value_of("INPUT").unwrap()))?;
    let input_file = macros::locate_file(input_file, &paths)?;
    let output_file = renderer
        .generate_output_filename(Path::new(args.value_of("OUTPUTDIR").unwrap()), &input_file);

    println!("input file: {:?}", input_file);
    println!("output file: {:?}", output_file);

    let exercise = Exercise::load_exercise(&input_file, paths)?;

    let render = exercise.render(renderer);

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
