// Expressions
//

//use lazy_static::lazy_static;
//use regex::Regex;
use std::collections::HashMap;

use crate::magnitude::Magnitude;

pub type DictVariables = HashMap<String, Expression>;

// String values are future improvements ¿except variable?

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Variable {
        name: String,
        value: Box<Expression>,
    },
    Magnitude(Magnitude),
    //formula(String),
    //algebra(String)
}

impl Expression {
    pub fn from(string: &str, global: &DictVariables, local: &DictVariables) -> Self {
        // variable?
        if let Some(value) = global.get(string) {
            return Expression::Variable {
                name: string.to_string(),
                value: Box::new(value.clone()),
            };
        }

        if let Some(value) = local.get(string) {
            return Expression::Variable {
                name: string.to_string(),
                value: Box::new(value.clone()),
            };
        }

        // magnitude?
        if let Some(value) = Magnitude::get(string) {
            return Expression::Magnitude(value);
        }

        // I don't know
        panic!("Don't know how to build Expression from: {:?}", string)
    }

    pub fn value(&self) -> String {
        match self {
            Expression::Variable { value, .. } => value.value(),
            Expression::Magnitude(value) => format!("{}", value.value),
        }
    }

    pub fn magnitude(&self) -> String {
        match self {
            Expression::Variable { value, .. } => value.magnitude(),
            Expression::Magnitude(value) => format!("{}", value),
        }
    }

    pub fn desc(&self) -> String {
        match self {
            Expression::Variable { name, .. } => name.clone(),
            Expression::Magnitude(value) => format!("{}", value),
        }
    }
    pub fn desc_magnitude(&self) -> String {
        match self {
            Expression::Variable { name, value } => format!("{}={}", name, value.desc_magnitude()),
            Expression::Magnitude(value) => format!("{}", value),
        }
    }
}
