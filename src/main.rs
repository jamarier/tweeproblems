use anyhow::{bail, Result};
use clap::{App, Arg};
//use std::fs::{write, File};
use std::path::{Path, PathBuf};
use uuid::Uuid;

mod expression;
mod formulas;
mod magnitude;
mod passage;

//use crate::expression::DictVariables;
use crate::passage::{Exercise};

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
        .get_matches();

    let input_file = check_input_file(Path::new(args.value_of("INPUT").unwrap()))?;
    let output_file = generate_output_filename(input_file);

    println!("input file: {:?}", input_file);
    println!("output file: {:?}", output_file);


    let exercise = Exercise::load_exercise(&input_file)?;

    let render = exercise.render();
    println!("\nrender {}",render);

    Ok(())
    /*
    // Open the file in read-only mode (ignoring errors).
    let mut line_iterator = open_input_file(input_file)?;

    let document_title = locate_title(&mut line_iterator)?;
    let mut output_lines: Vec<String> = preface(&document_title);





    let mut passage_title = PassageTitles::new();

    let mut variables: DictVariables = DictVariables::new();

    let mut previous_text = String::new();

    while let Some(passage) = Passage::read_passage(&mut line_iterator, &mut variables) {
        previous_text = previous_text + "\n" + &passage.text;
        let passage = Passage {
            text: previous_text.clone(),
            options: passage.options,
            aftertext: passage.aftertext,
            aftervariables: passage.aftervariables,
        };

        output_lines.push(passage.build_subtree(&mut passage_title));

        previous_text = previous_text + "\n" + &passage.aftertext;

        variables.extend(passage.aftervariables.clone());
        passage_title.inc_chapter();
    }

    write(output_file, output_lines.join("\n"))?;

    Ok(())
    */
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

