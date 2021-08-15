// Passage generation

use anyhow::{Result};
use lazy_static::lazy_static;
//use rand::seq::SliceRandom;
use regex::Regex;
//use std::fmt;
use std::fs;
//use std::fs::File;
//use std::io::BufReader;
use yaml_rust::{Yaml,YamlLoader};
use yaml_rust::yaml::{Hash};
use std::path::Path;

use crate::expression::{DictVariables, Expression};

// Gate: info about an option
#[derive(Debug, Clone)]
pub struct Gate {
    text: String,             // ___ text to show to be able to make a choice
    follow: String,           // ... text shown after election in the main history
    note: String,             // --- text shown after election in a temporal history
    variables: DictVariables, // variables defined after this gate.
}

impl Gate {
    fn from(string : &str, variables: &DictVariables) -> Self {
        println!("Gate::From {:?}", string);

        let mut text = Vec::<String>::new();
        let mut follow = Vec::<String>::new();
        let mut note = Vec::<String>::new();

        let mut variables = variables.clone();

        let mut status = GateStatus::Text;

        let lines = string.split("\n");
        for mut line in lines {
            if let Some(verbatim) = line_start_with("!",line) {
                line = verbatim;
                match status {
                    GateStatus::Text => text.push(line.to_string()),
                    GateStatus::Follow => follow.push(line.to_string()),
                    GateStatus::Note => note.push(line.to_string()),
                }
                continue;
            }
            
            if let Some(rest) = line_start_with("___",line) {
                status = GateStatus::Text;
                line = rest;
            } else if let Some(rest) = line_start_with("...",line) {
                status = GateStatus::Follow;
                line = rest;
            } else if let Some(rest) = line_start_with("---",line) {
                status = GateStatus::Note;
                line = rest;
            }

            match status {
                GateStatus::Text => text.push(process_line(&line, &mut variables)),
                GateStatus::Follow => follow.push(process_line(&line, &mut variables)),
                GateStatus::Note => note.push(process_line(&line, &mut variables)),
            }
        }

        Gate {text: text.join("\n"), follow: follow.join("\n"), note: note.join("\n"), variables: variables}

    }
}

fn line_start_with<'a,'b>( preffix: &'a str, line : &'b str) -> Option<&'b str> {
    if line.starts_with(preffix) {
        Some(&line[preffix.len()..])
    } else {
        None
    }
}

enum GateStatus {
    Text,
    Follow,
    Note
}

//-------------------------
fn process_line(
    line: &str,
    vars: &mut DictVariables,
) -> String {
    let mut output_vec = Vec::<String>::new();

    let line = encode_line(&line);

    lazy_static! {
        static ref RE_INTERPOLATION: Regex = Regex::new(
            r"(?x)                 # extended mode
               \{\{                # initial parantheses 
               (.)                 # 1 code for interpolation type
               \s*                  
               ( ([[:^blank:]]+?) \s* = \s* )?    # 2 3 possible binding
               (.+?)               # 4 definition
               \s*
               \}\}
               "
        )
        .unwrap();
    }

    // displaymode
    let (start_math, end_math) = marker_math(is_displaymode(&line));

    // it is a cursor/offset in line
    let mut it: usize = 0;
    
    for cap in RE_INTERPOLATION.captures_iter(&line) {
        // pass everythin before interpolation
        let m = cap.get(0).unwrap();
        output_vec.push(decode_line(&line[it..m.start()]));

        //println!("\n\nreading line: {:?}", &line);
        let value: Expression = Expression::from(&decode_line(&cap[4]));
        //println!("Expression: {:?}", value);

        // binding
        let var_name: String = match cap.get(3) {
            Some(var) => decode_line(var.as_str()),
            None => String::new(),
        };

        if !var_name.is_empty() {
            // there is binding

            match vars.get(&var_name) {
                Some(value_dict) => {
                    let v1 = value.value(vars);
                    let v2 = value_dict.value(vars);

                    if (v1.value - v2.value).abs() > 1e-5 || v1.unit != v2.unit {
                        panic!("Attempt of overwrite variable: {:?}. Old value: {:?}={} and new value: {:?}={}", var_name, value_dict, value_dict.value(vars), value, value.value(vars));
                    }
                }
                None => {
                    vars.insert(var_name.clone(), value.clone());
                }
            }
        }

        // printing/inyecting
        match &cap[1] {
            "." => {
                // Shows only the value
                output_vec.push(start_math.to_string());
                if !var_name.is_empty() {
                    output_vec.push(format!("{} = ", var_name));
                }
                output_vec.push(format!("{}", value.value(&vars)));
                output_vec.push(end_math.to_string());
            }
            "," => {
                // show only the calculation
                output_vec.push(start_math.to_string());
                if !var_name.is_empty() {
                    output_vec.push(format!("{} = ", var_name));
                }
                output_vec.push(value.show());
                output_vec.push(end_math.to_string());
            }
            ";" => {
                // calculation and later the value
                output_vec.push(start_math.to_string());
                if !var_name.is_empty() {
                    output_vec.push(format!("{} = ", var_name));
                }
                output_vec.push(value.show());
                output_vec.push(String::from(" = "));
                output_vec.push(format!("{}", value.value(&vars)));
                output_vec.push(end_math.to_string());
            }
            "!" => {
                // Inject numeric value
                if !var_name.is_empty() {
                    output_vec.push(format!("{}=", var_name));
                }
                output_vec.push(format!("{}", value.value(&vars).value));
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

fn is_displaymode(line: &str) -> bool {
    lazy_static! {
        static ref RE_ISDISPLAY: Regex = Regex::new(
        r"(?xi)
            ^ 
            [^[:alnum:]]*     
            \{\{
            (.*)               # 1
            \}\}
            [^[:alnum:]]*
            $
        ",
        )
        .unwrap();
    }

    match RE_ISDISPLAY.captures(line) {
        Some(matches) => !matches[1].contains("}}"),
        None => false,
    }
}

fn marker_math(displaymode: bool) -> (&'static str, &'static str) {
    if displaymode {
        ("\"\"\"\\[ ", " \\]\"\"\"")
    } else {
        ("\"\"\"\\( ", " \\)\"\"\"")
    }
}

//-------------------------

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


//--------------------------------------------------------------------------------

#[derive(Debug,Clone)]
pub struct Passage {
    previous_bad: Vec<Gate>,
    text: Gate,
    post_bad: Vec<Gate>,
}

#[derive(Debug,Clone)]
enum PassageElem {
    None,
    Passage(Passage),
    Sequential(Vec<PassageElem>),
    Concurrent(Vec<PassageElem>),
    Alternative(Vec<PassageElem>),
}

pub struct Exercise {
    title: String,
    passages: PassageElem
}

pub fn load_exercise(file: &Path) -> Result<Exercise> {
    let contents = fs::read_to_string(file).expect("Unable to read file");
    let docs = YamlLoader::load_from_str(&contents).unwrap();
    let doc = &docs[0];

    let variables = DictVariables::new();

    let title = doc["title"].as_str().unwrap().to_owned();
    println!("\ntitle: {:?}",title);

    println!("\ndocs: {:?}", doc);
    let passages = convert_yaml(&doc["passages"], variables);
    println!("\npassages: {:?}", passages);
    

    Ok(Exercise { title, passages: passages.0 })
}

fn unique_key(hash: &Hash) -> Option<&str> {
    if hash.len() != 1 {
        return None;
    }

    return Some(hash.front().unwrap().0.as_str().unwrap());
}

fn convert_yaml(yaml: &Yaml, dictionary: DictVariables) -> (PassageElem, DictVariables) {
    let (output,vars) = match yaml {
        Yaml::Hash(hash) => match unique_key(hash) {
                Some("pass") => convert_pass(&yaml["pass"],dictionary),
                Some("seq") => convert_seq(yaml["seq"].as_vec().unwrap(),dictionary),
                Some("alt") => convert_seq(yaml["alt"].as_vec().unwrap(),dictionary),
                Some("con") => convert_seq(yaml["con"].as_vec().unwrap(),dictionary),
                _ => panic!("I don't know how to process {:?}", hash)
            }
        _ => panic!("I don't know how to process {:?}", yaml)
    };

    (output, vars)
}

fn convert_pass(pass: &Yaml, dictionary: DictVariables) -> (PassageElem, DictVariables) {
    println!("\npass: {:?}", pass);

    let text = Gate::from(pass["text"].as_str().unwrap(), &dictionary);

    let previous_bad = vec![];
    let post_bad = vec![];

    let vars = text.variables.clone();

    println!("\n text: {:?}", text);
    (PassageElem::Passage(Passage{previous_bad, text, post_bad}), vars)
}

fn convert_seq(elems: &Vec<Yaml>, dictionary: DictVariables) -> (PassageElem, DictVariables) {
    let mut dict = dictionary.clone();
    let mut passages = Vec::<PassageElem>::new();

    println!("\nelems: {:?}", elems);
    for elem in elems {
        println!("\nelem: {:?}", elem);
        let (passelem, ndict) = convert_yaml(elem, dict);
        dict = ndict;
        passages.push(passelem)
    }
    (PassageElem::Sequential(passages), dict)
}



/*


// --------------------------------------------------------------------------------
// Old content

// Some constants

static END_PASSAGE: &str = ">>>";
static START_GOOD_GATE: &str = "-->";
static START_BAD_GATE: &str = "--X";
static LENGTH_START: usize = 3;
static START_EXPLANATION: &str = "--";
static LENGTH_EXPLANATION: usize = 2;

// Passage is an structure of:
// * text to describe the situation until that moment
// * a set of possible next steps (some true and some false)
// * a mark of end of passage.


#[derive(Debug, PartialEq, Eq)]
enum ReadingPassageStatus {
    Text,
    GateText,
    GateExplanation,
}

#[derive(Debug)]
pub struct Passage {
    pub text: String,
    pub options: Vec<Gate>,
    // all info
    pub aftertext: String,
    pub aftervariables: DictVariables,
}

impl Passage {
    pub fn read_passage(
        line_iterator: &mut std::io::Lines<BufReader<File>>,
        variables: &mut DictVariables,
    ) -> Option<Passage> {
        let mut line_string: String;

        // text_mode or gates_mode
        let mut reading_status = ReadingPassageStatus::Text;

        // info to text mode
        let mut lines_text = Vec::<String>::new();

        // info to gates mode
        let mut gates = Vec::<Gate>::new();
        let mut local_variables = DictVariables::new();
        let mut lines_gate = Vec::<String>::new();
        let mut lines_explanation = Vec::<String>::new();
        let mut good_gate: bool = false;

        // After info
        let mut aftertext = String::new();
        let mut aftervariables = DictVariables::new();

        // reading lines of text
        for line in line_iterator {
            line_string = line.unwrap();

            // line processing

            if !line_string.is_empty() {
                let first_char = line_string.chars().next().unwrap();
                if first_char == '!' {
                    line_string = line_string[1..].trim().to_string();
                } else {
                    // line to process
                    line_string = process_line(
                        line_string,
                        reading_status == ReadingPassageStatus::Text,
                        variables,
                        &mut local_variables,
                    );
                }
            }

            // End of passage
            //    The order of if clauses is relevant and dependent on constants used
            if line_string.starts_with(END_PASSAGE) {
                break;
            } else if line_string.starts_with(START_GOOD_GATE)
                || line_string.starts_with(START_BAD_GATE)
            {
                // Start of Gate
                reading_status = ReadingPassageStatus::GateText;

                // Create new gate
                if !lines_gate.is_empty() {
                    if good_gate {
                        let temp = local_variables.clone();
                        aftervariables.extend(temp);
                    }
                    let last_gate = Gate {
                        text: lines_gate.join("\n"),
                        explanation: lines_explanation.join("\n"),
                        good: good_gate,
                        variables: local_variables,
                    };
                    gates.push(last_gate);

                    lines_gate = vec![];
                    lines_explanation = vec![];
                    local_variables = DictVariables::new();
                }

                // next gate title and type creation
                let start = &line_string[0..LENGTH_START];
                if start == START_GOOD_GATE {
                    good_gate = true;
                    let temp = line_string[LENGTH_START..].trim().to_string();
                    aftertext = aftertext + &temp + "\n";
                    lines_gate.push(temp);
                } else {
                    good_gate = false;
                    lines_gate.push(line_string[LENGTH_START..].trim().to_string());
                }
            } else if line_string.starts_with(START_EXPLANATION) {
                reading_status = ReadingPassageStatus::GateExplanation;
                lines_explanation.push(line_string[LENGTH_EXPLANATION..].trim().to_string());
            } else {
                // adding lines
                match reading_status {
                    ReadingPassageStatus::Text => lines_text.push(line_string),
                    ReadingPassageStatus::GateText => {
                        if good_gate {
                            aftertext = aftertext + &line_string + "\n";
                        }
                        lines_gate.push(line_string);
                    }
                    ReadingPassageStatus::GateExplanation => lines_explanation.push(line_string),
                }
            }
        }

        // adding last gate
        if !lines_gate.is_empty() {
            if good_gate {
                let temp = local_variables.clone();
                aftervariables.extend(temp);
            }

            let last_gate = Gate {
                text: lines_gate.join("\n"),
                explanation: lines_explanation.join("\n"),
                good: good_gate,
                variables: local_variables,
            };
            gates.push(last_gate);
        }

        if !lines_text.is_empty() || !gates.is_empty() {
            Some(Passage {
                text: lines_text.join("\n"),
                options: gates,
                aftertext,
                aftervariables,
            })
        } else {
            None
        }
    }

    fn count_correct_gates(&self) -> u16 {
        let mut count = 0;

        for option in &self.options {
            if option.good {
                count += 1;
            }
        }

        count
    }

    pub fn build_subtree(&self, base_title: &mut PassageTitles) -> String {
        let mut suboutput: Vec<String> = vec![];

        // main text of passage
        let mut output: Vec<String> = vec![format!(":: {}\n", base_title), self.text.clone()];

        // last passage
        if self.options.is_empty() {
            //TODO: I18N
            output.push(String::from("\n[[Volver a empezar -> Start]] \n"));
        } else {
            //TODO: I18N
            output.push(String::from("\nMarque una opción correcta:\n"));
        }

        // shuffle of gates
        let mut gates = self.options.clone();
        gates.shuffle(&mut rand::thread_rng());

        let current_link = base_title.to_string();
        let last_good_correct = self.count_correct_gates() == 1;

        for (it, gate) in gates.iter().enumerate() {
            if gate.good && last_good_correct {
                if !gate.explanation.is_empty() {
                    let explanation_link = base_title.inc_section();
                    output.push(build_option_msg(it + 1, &explanation_link, &gate.text));
                    let next_link = base_title.show_next_chapter();
                    suboutput.push(build_explanation_gate(
                        &explanation_link,
                        &gate.explanation,
                        "Continuar",
                        &next_link,
                    ));
                } else {
                    // good last without explanations
                    let next_link = base_title.show_next_chapter();
                    output.push(build_option_msg(it + 1, &next_link, &gate.text));
                }
            } else if gate.good {
                if !gate.explanation.is_empty() {
                    let explanation_link = base_title.inc_section();
                    output.push(build_option_msg(it + 1, &explanation_link, &gate.text));
                    let next_link = base_title.inc_section();
                    suboutput.push(build_explanation_gate(
                        &explanation_link,
                        &gate.explanation,
                        "Continuar",
                        &next_link,
                    ));
                } else {
                    // without explanations
                    let next_link = base_title.inc_section();
                    output.push(build_option_msg(it + 1, &next_link, &gate.text));
                }

                // building subtree
                let new_text = format!("{}\n{}\n", self.text.clone(), gate.text);
                let mut new_gates = gates.clone();
                new_gates.remove(it);

                suboutput.push(
                    Passage {
                        text: new_text,
                        options: new_gates,
                        aftertext: String::new(),
                        aftervariables: DictVariables::new(),
                    }
                    .build_subtree(base_title),
                );
            } else {
                // bad gate
                base_title.inc_section();
                let explanation_title = base_title.to_string();
                output.push(build_option_msg(it + 1, &explanation_title, &gate.text));
                suboutput.push(build_explanation_gate(
                    &explanation_title,
                    &gate.explanation,
                    "Intentarlo nuevamente",
                    &current_link,
                ));
            }
        }

        output.append(&mut suboutput);

        output.join("\n")
    }
}

//-------------------------

fn build_option_msg(order: usize, next_link: &str, text: &str) -> String {
    //TODO I18N
    format!("* [[Opción {}: -> {}]]\n{}", order, next_link, text)
}

fn build_explanation_gate(
    current_link: &str,
    explanation: &str,
    mesg: &str,
    next_link: &str,
) -> String {
    format!(
        "\n:: {}\n{}\n[[{} -> {}]]\n",
        current_link, explanation, mesg, next_link
    )
}

//-------------------------

#[derive(Debug, Clone)]
pub struct PassageTitles {
    chapter: u16,
    section: u16,
}

impl PassageTitles {
    pub fn new() -> Self {
        PassageTitles {
            chapter: 0,
            section: 0,
        }
    }

    pub fn inc_chapter(&mut self) {
        self.chapter += 1;
        self.section = 0;
    }

    pub fn show_next_chapter(&self) -> String {
        let mut temp = self.clone();
        temp.inc_chapter();
        temp.to_string()
    }

    pub fn inc_section(&mut self) -> String {
        self.section += 1;
        self.to_string()
    }
}

impl fmt::Display for PassageTitles {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let base: String = if self.chapter == 0 {
            String::from("Start")
        } else {
            format!("Chapter-{}", self.chapter)
        };

        if self.section == 0 {
            write!(formatter, "{}", base)
        } else {
            write!(formatter, "{}_{}", base, self.section)
        }
    }
}

*/
