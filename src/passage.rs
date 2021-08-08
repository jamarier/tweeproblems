// Passage generation

use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::BufReader;

use crate::expression::{DictVariables, Expression};

// Some constants

static END_PASSAGE: &str = "---";
static START_GOOD_GATE: &str = "->";
static START_BAD_GATE: &str = "-X";

// Passage is an structure of:
// * text to describe the situation until that moment
// * a set of possible next steps (some true and some false)
// * a mark of end of passage.

// Gate: info about an option
#[derive(Debug)]
struct Gate {
    title: String,
    text: String,
    good: bool,
    variables: DictVariables,
}

impl Gate {}

#[derive(Debug)]
pub struct Passage {
    text: String,
    options: Vec<Gate>,
}

impl Passage {
    pub fn read_passage(
        line_iterator: std::io::Lines<BufReader<File>>,
        variables: &mut DictVariables,
    ) -> Passage {
        println!("Reading passage");

        let mut line_string: String;

        // text_mode or gates_mode
        let mut reading_gates: bool = false;

        // info to text mode
        let mut lines_text = Vec::<String>::new();

        // info to gates mode
        let mut gates = Vec::<Gate>::new();
        let mut local_variables = DictVariables::new();
        let mut lines_gate = Vec::<String>::new();
        let mut gate_title: String = String::new();
        let mut good_gate: bool = false;

        // reading lines of text
        for line in line_iterator {
            line_string = line.unwrap();

            // line processing

            if !line_string.is_empty() {
                if &line_string[0..1] == "!" {
                    line_string = line_string[1..].trim().to_string();
                } else {
                    // line to process
                    line_string =
                        process_line(line_string, !reading_gates, variables, &mut local_variables);
                }
            }

            // End of passage
            if line_string.starts_with(END_PASSAGE) {
                break;
            } else if line_string.starts_with(START_GOOD_GATE)
                || line_string.starts_with(START_BAD_GATE)
            {
                // Start of Gate
                reading_gates = true;

                // Create new gate
                if !lines_gate.is_empty() {
                    let last_gate = Gate {
                        title: gate_title,
                        text: lines_gate.join("\n"),
                        good: good_gate,
                        variables: local_variables,
                    };
                    gates.push(last_gate);

                    lines_gate = vec![];
                    local_variables = DictVariables::new();
                }

                // next gate title and type creation
                gate_title = line_string[2..].trim().to_string();
                let start = &line_string[0..2];
                if start == START_GOOD_GATE {
                    good_gate = true;
                } else {
                    good_gate = false;
                }
            } else {
                // adding lines
                if reading_gates {
                    lines_gate.push(line_string);
                } else {
                    lines_text.push(line_string);
                }
            }
        }

        // adding last gate
        if !lines_gate.is_empty() {
            let last_gate = Gate {
                title: gate_title,
                text: lines_gate.join("\n"),
                good: good_gate,
                variables: local_variables,
            };
            gates.push(last_gate);
        }

        println!("Passage text: {}", lines_text.join("\n"));

        Passage {
            text: lines_text.join("\n"),
            options: gates,
        }
    }
}

fn encode_line(string: &str) -> String {
    string
        .replace("\\\\", "\\0")
        .replace("\\{", "\\a")
        .replace("\\}", "\\b")
}

fn decode_line(string: &str) -> String {
    string
        .replace("\\b", "}")
        .replace("\\a", "{")
        .replace("\\0", "\\")
}

fn process_line(
    line: String,
    to_global: bool,
    global_vars: &mut DictVariables,
    local_vars: &mut DictVariables,
) -> String {
    let mut output_vec = Vec::<String>::new();

    let line = encode_line(&line);

    lazy_static! {
        static ref RE_INTERPOLATION: Regex = Regex::new(
            r"(?x)              # extended mode
                       \{\{                # initial parantheses 
                       (.)                 # code for interpolation type
                       \s*                  
                       ( (.+?) \s* = \s* )?    # possible binding
                       (.+?)               # definition
                       \s*
                       \}\}
                       "
        )
        .unwrap();
    }

    // search for inline code
    let mut it: usize = 0;
    for cap in RE_INTERPOLATION.captures_iter(&line) {
        // pass everythin before interpolation
        let m = cap.get(0).unwrap();
        output_vec.push(decode_line(&line[it..m.start()]));

        println!("\n\nreading line: {:?}", &line);
        let value: Expression = Expression::from(&decode_line(&cap[4]));
        println!("Expression: {:?}", value);

        // binding
        let var_name: String = match cap.get(3) {
            Some(var) => decode_line(var.as_str()),
            None => String::new(),
        };

        if !var_name.is_empty() {
            // there is binding

            if to_global {
                match global_vars.get(&var_name) {
                    Some(value_dict) => {
                        if value != *value_dict {
                            panic!("Variable: {:?} has value: {:?} in global dictionary but {:?} is new value", var_name, value_dict, value);
                        }
                    }
                    None => {
                        global_vars.insert(var_name.clone(), value.clone());
                    }
                }
            } else {
                match local_vars.get(&var_name) {
                    Some(value_dict) => {
                        if value != *value_dict {
                            panic!("Variable: {:?} has value: {:?} in local dictionary but {:?} is new value", var_name, value_dict, value);
                        }
                    }
                    None => {
                        local_vars.insert(var_name.clone(), value.clone());
                    }
                }
            }
        }

        // printing/inyecting
        match &cap[1] {
            "." => {
                output_vec.push(String::from("\\( "));
                if !var_name.is_empty() {
                    output_vec.push(format!("{} = ", var_name));
                }
                output_vec.push(format!("{}", value.value(&global_vars, &local_vars)));
                output_vec.push(String::from(" \\)"));
            }
            "," => {
                output_vec.push(String::from("\\( "));
                if !var_name.is_empty() {
                    output_vec.push(format!("{} = ", var_name));
                }
                output_vec.push(value.show());
                output_vec.push(String::from(" \\)"));
            }
            ";" => {
                output_vec.push(String::from("\\( "));
                if !var_name.is_empty() {
                    output_vec.push(format!("{} = ", var_name));
                }
                output_vec.push(value.show());
                output_vec.push(String::from(" = "));
                output_vec.push(format!("{}", value.value(&global_vars, &local_vars)));
                output_vec.push(String::from(" \\)"));
            }
            "!" => {
                if !var_name.is_empty() {
                    output_vec.push(format!("{}=", var_name));
                }
                output_vec.push(format!("{}", value.value(&global_vars, &local_vars).value));
            }
            "_" => {} // Make calculation but doesn't show anything
            _ => {
                println!("Desconocido: \n\t{:?}", value);
            }
        }

        it = m.end();
    }
    if it < line.len() {
        output_vec.push(decode_line(&line[it..]));
    }

    // no match. all in_line into output
    if output_vec.is_empty() {
        output_vec.push(decode_line(&line));
    }

    output_vec.join("")
}
