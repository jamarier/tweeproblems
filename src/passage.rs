// Passage generation

use anyhow::{bail, Result};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use regex::Regex;
use std::fmt;
use std::fs;
//use std::fs::File;
//use std::io::BufReader;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use yaml_rust::yaml::Hash;
use yaml_rust::{Yaml, YamlLoader};

use crate::expression::{DictVariables, Expression};
use crate::macros::{include_macros, Macros};

// Gate: info about an option
#[derive(Debug, Clone)]
pub struct Gate {
    text: String,             // ___ text to show to be able to make a choice
    follow: String,           // ... text shown after election in the main history
    note: String,             // --- text shown after election in a temporal history
    variables: DictVariables, // variables defined after this gate.
}

impl Gate {
    fn from(string: &str, variables: &DictVariables, macros: &Macros) -> Self {
        let mut text = Vec::<String>::new();
        let mut follow = Vec::<String>::new();
        let mut note = Vec::<String>::new();

        let mut variables = variables.clone();

        let mut status = GateStatus::Text;

        let lines = string.split('\n');
        for mut line in lines {
            if let Some(verbatim) = line.strip_prefix('!') {
                line = verbatim;
                match status {
                    GateStatus::Text => text.push(line.to_string()),
                    GateStatus::Follow => follow.push(line.to_string()),
                    GateStatus::Note => note.push(line.to_string()),
                }
                continue;
            }

            if let Some(rest) = line.strip_prefix("___") {
                status = GateStatus::Text;
                line = rest;
            } else if let Some(rest) = line.strip_prefix("...") {
                status = GateStatus::Follow;
                line = rest;
            } else if let Some(rest) = line.strip_prefix("---") {
                status = GateStatus::Note;
                line = rest;
            }

            match status {
                GateStatus::Text => text.push(process_line(&line, &mut variables, &macros)),
                GateStatus::Follow => follow.push(process_line(&line, &mut variables, &macros)),
                GateStatus::Note => note.push(process_line(&line, &mut variables, &macros)),
            }
        }

        Gate {
            text: text.join("\n"),
            follow: follow.join("\n"),
            note: note.join("\n"),
            variables,
        }
    }

    fn passage_note(
        &self,
        current_link: &PassageTitle,
        msg: &str,
        output_link: &PassageTitle,
    ) -> String {
        format!(
            "\n:: {}\n\n{}\n\n[[ {} -> {} ]]\n\n\n",
            current_link, self.note, msg, output_link
        )
    }

    fn passage_bad_note(
        &self,
        current_link: &PassageTitle,
        msg: &str,
        output_link: &PassageTitle,
    ) -> String {
        format!(
            "\n:: {}\n\n''Opción errónea''\n\n{}\n\n[[ {} -> {} ]]\n\n\n",
            current_link, self.note, msg, output_link
        )
    }

    fn passage_choice(&self, next_link: &PassageTitle) -> String {
        // TODO I18N
        format!("[[ Opción: -> {} ]]\n\n{}\n\n", next_link, self.text)
    }

    fn has_note(&self) -> bool {
        !self.note.is_empty()
    }
}

enum GateStatus {
    Text,
    Follow,
    Note,
}

//-------------------------
fn process_line(line: &str, vars: &mut DictVariables, macros: &Macros) -> String {
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
        let value: Expression = Expression::from(&decode_line(&cap[4]), macros);
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

#[derive(Debug, Clone)]
pub struct Passage {
    previous_bad: Vec<Gate>,
    text: Gate,
    post_bad: Vec<Gate>,
}

impl Passage {}

impl fmt::Display for Passage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P{{{}}}", self.text.text)
    }
}

#[derive(Debug, Clone)]
enum PassageElem {
    Passage(Passage),
    Sequence(Vec<PassageElem>),
    Concurrent(Vec<PassageElem>),
    Alternative(Vec<PassageElem>),
}

impl PassageElem {
    /*
    fn previous_bad(&self) -> Vec<Gate> {
        match self {
            PassageElem::Passage(p) => p.previous_bad.clone(),
            PassageElem::Sequence(v) => v.first().unwrap().previous_bad(),
            PassageElem::Concurrent(v) => {
                let mut output = vec![];
                for e in v {
                    output.extend(e.previous_bad());
                }
                output
            }
            PassageElem::Alternative(v) => {
                let mut output = vec![];
                for e in v {
                    output.extend(e.previous_bad());
                }
                output
            }
        }
    }

    fn gate_text(&self) -> Vec<Gate> {
        match self {
            PassageElem::Passage(p) => vec![p.text.clone()],
            PassageElem::Sequence(v) => v.first().unwrap().gate_text(),
            PassageElem::Concurrent(v) => {
                let mut output = vec![];
                for e in v {
                    output.extend(e.gate_text());
                }
                output
            }
            PassageElem::Alternative(v) => {
                let mut output = vec![];
                for e in v {
                    output.extend(e.gate_text());
                }
                output
            }
        }
    }

    fn post_bad(&self) -> Vec<Gate> {
        match self {
            PassageElem::Passage(p) => p.post_bad.clone(),
            PassageElem::Sequence(v) => v.last().unwrap().post_bad(),
            _ => vec![],
        }
    }
    */
}

#[derive(Debug, Clone)]
struct PassageTree(Passage, Vec<PassageTree>);

impl PassageTree {
    fn new(passage: Passage) -> Self {
        PassageTree(passage, vec![])
    }

    fn from(elem: &PassageElem) -> Vec<Self> {
        match elem {
            PassageElem::Passage(p) => vec![PassageTree::new(p.clone())],
            PassageElem::Sequence(v) => {
                let mut v = v.clone();
                let first: PassageElem = v.remove(0);
                let mut output = PassageTree::from(&first);

                for elem in v {
                    output = output
                        .into_iter()
                        .map(|line| concatenate_tree(line, &PassageTree::from(&elem)))
                        .collect();
                }
                output
            }
            PassageElem::Alternative(v) => {
                let mut output: Vec<PassageTree> = vec![];
                for elem in v {
                    output.extend(PassageTree::from(elem));
                }
                output
            }
            PassageElem::Concurrent(v) => {
                if v.len() == 1 {
                    return PassageTree::from(&v[0]);
                }

                let mut output: Vec<PassageTree> = vec![];

                for (it, elem) in v.iter().enumerate() {
                    let mut rest = v.clone();
                    rest.remove(it);
                    for sub_elem in PassageTree::from(elem) {
                        output.push(concatenate_tree(
                            sub_elem,
                            &PassageTree::from(&PassageElem::Concurrent(rest.clone())),
                        ));
                    }
                }

                output
            }
        }
    }

    fn render(&self, acumulated_text: &str, current_link: &PassageTitle) -> String {
        let mut output = format!(":: {}\n\n", current_link);
        let mut suboutput = String::new();

        let mut accumulated_text = acumulated_text.to_string();
        accumulated_text = accumulated_text + &self.0.text.text + "\n";
        accumulated_text = accumulated_text + &self.0.text.follow + "\n";

        output += &accumulated_text;

        // end of render
        if self.1.is_empty() {
            output += "\n[[ Este es el final del problema. Volver al inicio -> Start ]]\n\n";
            return output;
        }

        // output gates
        let mut output_gates: Vec<String> = vec![];

        // sub_links
        let mut sub_link = current_link.clone();
        sub_link.add_level();

        // bad_gates
        let mut bad_gates = self.0.post_bad.clone();
        for next in &self.1 {
            bad_gates.extend(next.0.previous_bad.clone());
        }

        for bad_gate in bad_gates {
            output_gates.push(bad_gate.passage_choice(&sub_link));
            // TODO I18N
            suboutput +=
                &bad_gate.passage_bad_note(&sub_link, "Volver a intentarlo", &current_link);

            sub_link.inc();
        }

        // good_gates
        for next in &self.1 {
            let gate = next.0.text.clone();
            output_gates.push(gate.passage_choice(&sub_link));
            if gate.has_note() {
                let temp_link = sub_link.clone();
                sub_link.inc();
                // I18N
                suboutput += &gate.passage_note(&temp_link, "Continuar", &sub_link);
            }

            suboutput += &next.render(&accumulated_text, &sub_link);
            sub_link.inc();
        }

        // randomize of gates and output

        if output_gates.len() > 1 {
            // TODO I18N
            output += "Marque una opción que considere correcta (puede haber más de una)\n\n";
            output_gates.shuffle(&mut rand::thread_rng());
        } else {
            // TODO I18N
            output += "Marque la opción indicada para continuar\n\n";
        }
        output += &output_gates.join("\n");

        output += &suboutput;
        output
    }
}

impl fmt::Display for PassageTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> [", self.0)?;

        for line in &self.1 {
            write!(f, "{},", line)?;
        }

        write!(f, "]")
    }
}

fn concatenate_tree(a: PassageTree, b: &[PassageTree]) -> PassageTree {
    if a.1.is_empty() {
        PassageTree(a.0, b.to_owned())
    } else {
        let subtree: Vec<PassageTree> =
            a.1.into_iter()
                .map(|line| concatenate_tree(line, b))
                .collect();
        PassageTree(a.0, subtree)
    }
}

pub struct Exercise {
    title: String,
    passage_tree: PassageTree,
}

impl Exercise {
    pub fn load_exercise(file: &Path) -> Result<Exercise> {
        let contents = fs::read_to_string(file).expect("Unable to read file");
        let docs = YamlLoader::load_from_str(&contents).unwrap();
        let doc = &docs[0];

        let variables = DictVariables::new();

        let macros = if let Some(macros_files) = is_macro(&doc) {
            include_macros(Macros::new(), &macros_files)
        } else {
            Macros::new()
        };

        let title = doc["title"].as_str().unwrap().to_owned();

        let passages = convert_yaml(&doc["passages"], &variables, &macros);

        let mut passage_trees = PassageTree::from(&passages.0);

        /*
        println!("\npassages: {:?}", passages);
        println!("\npassageTree: {:?}", passage_trees);
        for it in &passage_trees {
            println!("\npassageTree: {}", it);
        }
        */

        if passage_trees.len() != 1 {
            bail!("\nThe document in file {:?} doesn't start with an passage (it starts with alternative or concurrent group)", file);
        }

        Ok(Exercise {
            title,
            passage_tree: passage_trees.pop().unwrap(),
        })
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        let passage_title = PassageTitle::new();

        output += &preface(&self.title);
        output += &self.passage_tree.render(&String::new(), &passage_title);

        output
    }
}

fn preface(title: &str) -> String {
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
        title,
        Uuid::new_v4()
    );

    output
}

fn unique_key(hash: &Hash) -> Option<&str> {
    if hash.len() != 1 {
        return None;
    }

    return Some(hash.front().unwrap().0.as_str().unwrap());
}

fn is_macro(elem: &Yaml) -> Option<Vec<PathBuf>> {
    match &elem["macros"] {
        Yaml::String(s) => Some(vec![PathBuf::from(s)]),
        Yaml::Array(a) => {
            let mut vector: Vec<PathBuf> = vec![];
            for item in a {
                if let Yaml::String(s) = item {
                    vector.push(PathBuf::from(s));
                }
            }
            if vector.is_empty() {
                None
            } else {
                Some(vector)
            }
        }
        _ => None,
    }
}

fn convert_yaml(
    yaml: &Yaml,
    dictionary: &DictVariables,
    macros: &Macros,
) -> (PassageElem, DictVariables, Macros) {
    match yaml {
        Yaml::Array(_) => convert_seq(yaml.as_vec().unwrap(), dictionary, macros),
        Yaml::Hash(hash) => match unique_key(hash) {
            Some("pass") => convert_pass(&yaml["pass"], dictionary, macros),
            Some("seq") => convert_seq(yaml["seq"].as_vec().unwrap(), dictionary, macros),
            Some("alt") => convert_alt(yaml["alt"].as_vec().unwrap(), dictionary, macros),
            Some("con") => convert_con(yaml["con"].as_vec().unwrap(), dictionary, macros),
            Some("macros") => panic!("'macros' directive misplaced"),
            _ => panic!("I don't know how to process {:?}", hash),
        },
        _ => panic!("I don't know how to process {:?}", yaml),
    }
}

fn convert_pass(
    pass: &Yaml,
    dictionary: &DictVariables,
    macros: &Macros,
) -> (PassageElem, DictVariables, Macros) {
    let text = Gate::from(pass["text"].as_str().unwrap(), dictionary, macros);

    let mut previous_bad = vec![];
    let mut post_bad = vec![];

    if let Some(vec) = pass["pre_bad"].as_vec() {
        for item in vec {
            //I18N
            previous_bad.push(Gate::from(item.as_str().unwrap(), dictionary, macros));
        }
    }

    if let Some(vec) = pass["post_bad"].as_vec() {
        for item in vec {
            post_bad.push(Gate::from(item.as_str().unwrap(), dictionary, macros));
        }
    }

    let vars = text.variables.clone();
    (
        PassageElem::Passage(Passage {
            previous_bad,
            text,
            post_bad,
        }),
        vars,
        macros.clone(),
    )
}

//-----------

fn convert_seq(
    elems: &[Yaml],
    dictionary: &DictVariables,
    macros: &Macros,
) -> (PassageElem, DictVariables, Macros) {
    let mut dict = dictionary.clone();
    let mut mac = macros.clone();
    let mut passages = Vec::<PassageElem>::new();

    for elem in elems {
        if let Some(macros_files) = is_macro(&elem) {
            mac = include_macros(mac, &macros_files);
        } else {
            let (passelem, ndict, nmac) = convert_yaml(elem, &dict, &mac);
            dict = ndict;
            mac = nmac;
            passages.push(passelem);
        }
    }
    (PassageElem::Sequence(passages), dict, mac)
}

fn convert_con(
    elems: &[Yaml],
    dictionary: &DictVariables,
    macros: &Macros,
) -> (PassageElem, DictVariables, Macros) {
    let base_dict = dictionary.clone();
    let mut dict = dictionary.clone();
    let mut mac = macros.clone();
    let mut passages = Vec::<PassageElem>::new();

    for elem in elems {
        let (passelem, ndict, nmac) = convert_yaml(elem, &base_dict, &mac);
        dict.extend(ndict);
        mac = nmac;
        passages.push(passelem)
    }
    (PassageElem::Concurrent(passages), dict, mac)
}

fn convert_alt(
    elems: &[Yaml],
    dictionary: &DictVariables,
    macros: &Macros,
) -> (PassageElem, DictVariables, Macros) {
    let base_dict = dictionary.clone();
    let mut dict = dictionary.clone();
    let mut mac = macros.clone();
    let mut passages = Vec::<PassageElem>::new();

    for elem in elems {
        let (passelem, ndict, nmac) = convert_yaml(elem, &base_dict, &mac);
        dict = ndict;
        mac = nmac;
        passages.push(passelem)
    }
    // dict is the dictionary of last branch
    (PassageElem::Alternative(passages), dict, mac)
}

//-------------------------

#[derive(Debug, Clone)]
pub struct PassageTitle(Vec<u16>);

impl PassageTitle {
    pub fn new() -> Self {
        PassageTitle(vec![0])
    }

    pub fn inc(&mut self) {
        let number = self.0.pop().unwrap();

        self.0.push(number + 1);
    }

    pub fn add_level(&mut self) {
        self.0.push(0);
    }
}

impl fmt::Display for PassageTitle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.len() == 1 && self.0[0] == 0 {
            write!(f, "Start")
        } else {
            let mut output = String::from("Chapter");
            for digit in &self.0[1..] {
                output = format!("{}-{}", output, digit + 1);
            }
            write!(f, "{}", output)
        }
    }
}
