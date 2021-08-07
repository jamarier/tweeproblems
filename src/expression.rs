// Expressions
//

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

use crate::magnitude::Magnitude;

pub type DictVariables = HashMap<String, Expression>;

// String values are future improvements ¿except variable?

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Empty,
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
            return Expression::Magnitude(value)
        }

        // I don't know
        Expression::Empty
    }
}
