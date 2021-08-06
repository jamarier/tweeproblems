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
use std::collections::HashMap;
use std::fmt;

// TABLES

lazy_static! {
// Factors
            static ref FACTORS : HashMap<char,f32> = hashmap!{
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

#[derive(Debug)]
pub struct Magnitude {
    value: f32,
    unit: String,
}

impl Magnitude {
    pub fn new(value: f32, unit: String) -> Self {
        (Magnitude { value, unit }).normalize()
    }

    // Normalize
    // TODO: add other factors
    fn normalize(self: Self) -> Self {
        let mut value = self.value;
        let mut unit = self.unit;

        if unit.len() > 1 {
            let units: Vec<char> = unit.chars().collect();
            let first = units[0];
            let second = units[1];

            if (second >= 'a' && second <= 'z') || (second >= 'A' && second <= 'Z') {
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
        let mut scaled: f32 = 0.0;
        let mut new_unit: String;

        if let Some(pretty) = UNITS.get(&*self.unit) {
            new_unit = (*pretty).to_owned();
        } else {
            new_unit = self.unit.clone();
        }

        // not very elegant but, works!
        for (factor_name, factor_value) in FACTORS.iter() {
            scaled = self.value / factor_value;
            if 1.0 <= scaled && scaled < 1e3 {
                if *factor_name != '1' {
                    new_unit = format!("{}{}", factor_name, new_unit);
                }
                break;
            }
        }

        write!(formatter, "{}\\mathrm{{{}}}", scaled, new_unit)
    }
}
