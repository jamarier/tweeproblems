// Structures to support the use of formulas in histories

use lazy_static::lazy_static;
use maplit::hashmap;
use std::collections::HashMap;

use crate::expression::Expression;

type Arguments = HashMap<String, Expression>;
type Parameters = HashMap<&'static str, &'static str>;

pub enum OhmLaw {
    OhmLawI { v: Expression, r: Expression },

    OhmLawV { i: Expression, r: Expression },

    OhmLawR { v: Expression, i: Expression },
}

lazy_static! {
    static ref OHMLAW_TYPES: Parameters = hashmap! {
        "R" => "ohm",
        "V" => "V",
        "I" => "I"
    };
}

impl OhmLaw {
    pub fn new(args: Arguments) -> Self {
        OhmLawI {
            v: Expression::Magnitude(Magnitude {
                value: 1,
                unit: String::from("ohm"),
            }),
            r: Expression::Magnitude(Magnitude {
                value: 1,
                unit: String::from("ohm"),
            }),
        }
    }
}
