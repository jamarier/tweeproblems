// Passage generation

use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io::BufReader;

use crate::expression::{DictVariables, Expression};

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

// Gate: info about an option
#[derive(Debug, Clone)]
pub struct Gate {
    text: String,
    explanation: String,
    good: bool,
    variables: DictVariables,
}

impl Gate {}

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

//-------------------------
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
    let start_math : &str;
    let end_math: &str;
    if is_displaymode(&line) {
        start_math = "\"\"\"\\[ ";
        end_math = " \\]\"\"\"";
    } else {
        start_math = "\"\"\"\\( ";
        end_math = " \\)\"\"\"";
    }

    // search for inline code
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
                // Shows only the value
                output_vec.push(start_math.to_string());
                if !var_name.is_empty() {
                    output_vec.push(format!("{} = ", var_name));
                }
                output_vec.push(format!("{}", value.value(&global_vars, &local_vars)));
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
                output_vec.push(format!("{}", value.value(&global_vars, &local_vars)));
                output_vec.push(end_math.to_string());
            }
            "!" => {
                // Inject numeric value
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
