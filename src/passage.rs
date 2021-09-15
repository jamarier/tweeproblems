// Passage generation

use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use regex::Regex;
use std::fmt;
use yaml_rust::yaml::Hash;
use yaml_rust::Yaml;

use crate::expression::{DictVariables, Expression};
use crate::macros::Macros;
use crate::render::Render;

// Gate: info about an option
#[derive(Debug, Clone)]
pub struct Gate {
    text: String,             // ___ text to show to be able to make a choice
    follow: String,           // ... text shown after election in the main history
    note: String,             // --- text shown after election in a temporal history
    variables: DictVariables, // variables defined after this gate.
}

impl Gate {
    fn new() -> Self {
        Gate {
            text: String::new(),
            follow: String::new(),
            note: String::new(),
            variables: DictVariables::new(),
        }
    }

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
                GateStatus::Text => text.push(process_line(line, &mut variables, macros)),
                GateStatus::Follow => follow.push(process_line(line, &mut variables, macros)),
                GateStatus::Note => note.push(process_line(line, &mut variables, macros)),
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
        renderer: &mut dyn Render,
        current_link: &PassageTitle,
        msg: &str,
        output_link: &PassageTitle,
    ) -> String {
        let mut output = String::new();

        output += &renderer.begin_passage(&current_link.to_string());
        output += &renderer.text(&self.note);
        output += &renderer.link(msg, &output_link.to_string());
        output += &renderer.end_passage(&current_link.to_string());

        output
    }

    fn passage_bad_note(
        &self,
        renderer: &mut dyn Render,
        current_link: &PassageTitle,
        msg: &str,
        output_link: &PassageTitle,
    ) -> String {
        let mut output = String::new();

        output += &renderer.begin_passage(&current_link.to_string());
        output += &renderer.text("Opción errónea");
        output += &renderer.text(&self.note);
        output += &renderer.link(msg, &output_link.to_string());
        output += &renderer.end_passage(&current_link.to_string());

        output
    }

    fn passage_choice(&self, renderer: &mut dyn Render, next_link: &PassageTitle) -> String {
        let mut output = String::new();
        output += &renderer.begin_option("", &next_link.to_string());
        output += &renderer.link("Opción:", &next_link.to_string());
        output += &renderer.text(&self.text);
        output += &renderer.end_option(&next_link.to_string());

        output
    }

    fn has_note(&self) -> bool {
        !self.note.is_empty()
    }

    fn is_empty(&self) -> bool {
        self.text.is_empty()
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

    let line = encode_line(line);

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
                output_vec.push(format!("{}", value.value(vars)));
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
                output_vec.push(format!("{}", value.value(vars)));
                output_vec.push(end_math.to_string());
            }
            "!" => {
                // Inject numeric value
                if !var_name.is_empty() {
                    output_vec.push(format!("{}=", var_name));
                }
                output_vec.push(format!("{}", value.value(vars).value));
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
        ("[[[ ", " ]]]")
    } else {
        ("((( ", " )))")
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

impl Passage {
    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

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
    /// extract text text from Gates in PassageElem
    fn text(&self) -> Vec<Gate> {
        match self {
            PassageElem::Passage(p) => vec![p.text.clone()],
            PassageElem::Sequence(s) => s[0].text(),
            PassageElem::Concurrent(s) => {
                let mut output = vec![];
                for p in s {
                    output.extend(p.text());
                }
                output
            }
            PassageElem::Alternative(s) => {
                let mut output = vec![];
                for p in s {
                    output.extend(p.text());
                }
                output
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PassageTree(Passage, Vec<PassageTree>);

impl PassageTree {
    fn new(passage: Passage) -> Self {
        PassageTree(passage, vec![])
    }

    pub fn from_yaml(yaml: &Yaml, dictionary: &DictVariables, macros: &Macros) -> Vec<Self> {
        let passages = convert_yaml(yaml, dictionary, macros);

        PassageTree::from(&passages.0)
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

    fn is_endnode(&self) -> bool {
        for node in &self.1 {
            if !node.0.is_empty() {
                return false;
            }
        }

        true
    }

    pub fn render(
        &self,
        renderer: &mut dyn Render,
        acumulated_text: &str,
        current_link: &PassageTitle,
    ) -> String {
        let mut output: String = renderer.begin_passage(&current_link.to_string());
        let mut suboutput = String::new();

        let mut acumulated_text = acumulated_text.to_string();
        acumulated_text = acumulated_text
            + &renderer.text(&self.0.text.text)
            + &renderer.text(&self.0.text.follow)
            + "\n";

        output += &acumulated_text;

        // end of render
        if self.is_endnode() {
            output += &renderer.begin_choices("");
            output += &renderer.link("Este es el final del problema. Volver al inicio", "Start");
            output += &renderer.end_choices("");
            output += &renderer.end_passage(&current_link.to_string());
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
            output_gates.push(bad_gate.passage_choice(renderer, &sub_link));
            // TODO I18N
            suboutput +=
                &bad_gate.passage_bad_note(renderer, &sub_link, "Volver a intentarlo", current_link);

            sub_link.inc();
        }

        // good_gates
        for next in &self.1 {
            let gate = next.0.text.clone();
            if gate.is_empty() {
                continue;
            }
            output_gates.push(gate.passage_choice(renderer, &sub_link));
            if gate.has_note() {
                let temp_link = sub_link.clone();
                sub_link.inc();
                // I18N
                suboutput += &gate.passage_note(renderer, &temp_link, "Continuar", &sub_link);
            }

            suboutput += &next.render(renderer, &acumulated_text, &sub_link);
            sub_link.inc();
        }

        // randomize of gates and output

        if output_gates.len() > 1 {
            // TODO I18N
            output += &renderer.begin_choices(
                "Marque una opción que considere correcta (puede haber más de una)",
            );
            output_gates.shuffle(&mut rand::thread_rng());
        } else {
            // TODO I18N
            output += &renderer.begin_choices("Marque la opción indicada para continuar");
        }
        output += &output_gates.join("\n");

        output += &renderer.end_choices("");
        output += &renderer.end_passage(&current_link.to_string());
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

fn main_key(hash: &Hash) -> Option<&str> {
    if hash.len() == 1 {
        return Some(hash.front().unwrap().0.as_str().unwrap());
    }

    if hash.contains_key(&Yaml::from_str("cond")) {
        return Some("cond");
    }

    None
}

pub fn is_macros(tag: &'static str, elem: &Yaml) -> Option<Vec<String>> {
    match &elem[tag] {
        Yaml::String(s) => Some(vec![s.to_string()]),
        Yaml::Array(a) => {
            let mut vector: Vec<String> = vec![];
            for item in a {
                if let Yaml::String(s) = item {
                    vector.push(s.to_string());
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
        Yaml::Hash(hash) => match main_key(hash) {
            Some("pass") => convert_pass(&yaml["pass"], dictionary, macros),
            Some("seq") => convert_seq(yaml["seq"].as_vec().unwrap(), dictionary, macros),
            Some("alt") => convert_alt(yaml["alt"].as_vec().unwrap(), dictionary, macros),
            Some("con") => convert_con(yaml["con"].as_vec().unwrap(), dictionary, macros),
            Some("cond") => convert_cond(&yaml["cond"], &yaml["cont"], dictionary, macros),
            Some("paths") => panic!("'paths' directive misplaced"),
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
        if let Some(paths) = is_macros("paths", elem) {
            mac.add_paths(paths);
        } else if let Some(macros_files) = is_macros("macros", elem) {
            mac.include_macros(macros_files);
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

fn convert_cond(
    cond: &Yaml,
    cont: &Yaml,
    dictionary: &DictVariables,
    macros: &Macros,
) -> (PassageElem, DictVariables, Macros) {
    let cond = Gate::from(cond.as_str().unwrap(), dictionary, macros);

    if cond.text == "1" {
        convert_yaml(cont, dictionary, macros)
    } else {
        let (passage_elem, _, _) = convert_yaml(cont, dictionary, macros);
        let text = passage_elem.text();
        (
            PassageElem::Passage(Passage {
                previous_bad: text,
                text: Gate::new(),
                post_bad: vec![],
            }),
            dictionary.clone(),
            macros.clone(),
        )
    }
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
