// Magnitude descriptions
//
// a number with units
//
// units are simply metadata, there is no verification in operations
// only operations:
//   normalize: move multiples to "fundamental". But only First unit:
//     Ex: 1km/s => 1000m/s, but 1m/ms is not 1000m/s
//   display: prettyprint in mathJax
//

use lazy_static::lazy_static;
use maplit::hashmap;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

// TYPES
type ValueType = f64;

// TABLES

lazy_static! {
// Regex
    static ref RE_MAGNITUDE: Regex =
        Regex::new(r"(?x)              # extended mode
                   (\d+                # 1 value: float
                    (\.\d+)?           # 2
                    (e\d+)?            # 3
                   )
                   \s*
                   (.*)                # 4 units
                   \s*
                   ").unwrap();


// Factors
            static ref FACTORS : HashMap<char,ValueType> = hashmap!{
                'T' => 1e12,
                'G' => 1e9,
                'M' => 1e6,
                'k' => 1e3,
                '1' => 1.0,
                'm' => 1e-3,
                'u' => 1e-6,
                'n' => 1e-9,
                'p' => 1e-12,
                'f' => 1e-15,
            };

// Units
            static ref UNITS: HashMap<&'static str,&'static str> = hashmap!{
                "ohm" => "\\Omega",
            };

        }

#[derive(Debug, Clone, PartialEq)]
pub struct Magnitude {
    pub value: ValueType,
    pub unit: String,
}

impl Magnitude {
    // try to process string into Magnitude.
    pub fn get(string: &str) -> Option<Self> {
        let without_underline = string.replace("_", "");
        RE_MAGNITUDE
            .captures(&without_underline)
            .map(|cap| Magnitude::new(cap[1].parse::<ValueType>().unwrap(), cap[4].to_string()))
    }

    pub fn new(value: ValueType, unit: String) -> Self {
        (Magnitude { value, unit }).normalize()
    }

    // Normalize
    // TODO: add other factors
    fn normalize(self) -> Self {
        let mut value = self.value;
        let mut unit = self.unit;

        if unit.len() > 1 {
            let units: Vec<char> = unit.chars().collect();
            let first = units[0];
            let second = units[1];

            if ('a'..='z').contains(&second) || ('A'..='Z').contains(&second) {
                if let Some(factor) = FACTORS.get(&first) {
                    value *= factor;
                    unit = unit[1..].to_owned();
                }
            }
        }

        Magnitude { value, unit }
    }
}

impl fmt::Display for Magnitude {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut scaled: ValueType = 0.0;
        let mut new_unit: String;

        if let Some(pretty) = UNITS.get(&*self.unit) {
            new_unit = (*pretty).to_owned();
        } else {
            new_unit = self.unit.clone();
        }

        let sign_str: &str;
        let value_abs: ValueType;

        if self.value > 0.0 {
            sign_str = "";
            value_abs = self.value;
        } else {
            sign_str = "-";
            value_abs = -1.0 * self.value;
        }

        // not very elegant but, works!
        for (factor_name, factor_value) in FACTORS.iter() {
            scaled = value_abs / factor_value;
            if (1.0..1e3).contains(&scaled) {
                if *factor_name != '1' {
                    new_unit = format!("{}{}", factor_name, new_unit);
                }
                break;
            }
        }

        scaled = (scaled * 100.0).round() / 100.0;
        if new_unit.is_empty() {
            write!(formatter, "{}{:}", sign_str, scaled)
        } else {
            write!(formatter, "{}{:}\\mathrm{{{}}}", sign_str, scaled, new_unit)
        }
    }
}
