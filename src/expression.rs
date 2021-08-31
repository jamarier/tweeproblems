// Expressions
//

use maplit::hashmap;
use rand::random;
use std::collections::HashMap;

use crate::macros::Macros;
use crate::magnitude::{self, Magnitude};

pub type DictVariables = HashMap<String, Expression>;
pub type Stack = Vec<Expression>;
pub type Argument = Box<Expression>;
pub type Arguments = Vec<Expression>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Magnitude(Magnitude),
    Variable(String),
    Add(Arguments),
    Neg(Argument),
    Prod(Arguments),
    Div(Arguments),
    Unit(Argument, String), // Assign Unit if unit = "¿?", verify unit if previus expression have unit
    Sqrt(Argument),
    Rand(Arguments),

    And(Arguments),
    Or(Arguments),
    Not(Argument),
    Eq(Arguments),
    Neq(Arguments),
    Le(Arguments),
    Leq(Arguments),
    Ge(Arguments),
    Geq(Arguments),
}

impl Expression {
    pub fn from(string: &str, macros: &Macros) -> Self {
        let mut stack: Stack = vec![];

        Expression::inject(string, &mut stack, macros);

        if stack.len() == 1 {
            stack[0].clone()
        } else {
            panic!(
                "Stack not empty at the end of expression analysis: {:?}",
                stack
            )
        }
    }

    fn inject(string: &str, stack: &mut Stack, macros: &Macros) {
        let mut dictionary: DictVariables = hashmap! {};

        let string: String = string
            .chars()
            .map(|x| match x {
                '\n' => ' ',
                '\t' => ' ',
                _ => x,
            })
            .collect();
        let items = string.split(' ');

        for current in items {
            if current.is_empty() {
            } else if let Some(magnitude) = Magnitude::get(current) {
                insert_magnitude(magnitude, stack);
            } else {
                match current {
                    // stack operators
                    "debug" => {
                        println!("*****\nDEBUG");
                        println!(" String: {:?}", string);
                        println!(" {:?}", stack);
                        println!(" {:?}", dictionary);
                        println!("END DEBUG\n*****\n");
                    }
                    "." => {
                        println!("*****\nTRACE");
                        println!(" String: {:?}", string);
                        println!(" Point {:?}", pop(stack).show());
                        println!("END TRACE\n*****\n");
                    }

                    // register operators
                    "!" => to_dict(stack, &mut dictionary),
                    "@" => from_dict(stack, &dictionary),

                    // units operators
                    ":" => operator2(units, stack),
                    "::" => operator1(nounits, stack),

                    // arithmetic operators
                    "+" => operator2(add_expression, stack),
                    "neg" => operator1(neg_expression, stack),
                    "-" => {
                        operator1(neg_expression, stack);
                        operator2(add_expression, stack);
                    }
                    "*" => operator2(prod_expression, stack),
                    "/" => operator2(div_expression, stack),
                    "sqrt" => operator1(sqrt_expression, stack),
                    "rand" => operator2(rand_expression, stack),

                    // logical and relational operators
                    "and" => operator2(and_expression, stack),
                    "or" => operator2(or_expression, stack),
                    "not" => operator1(not_expression, stack),
                    "==" => operator2(eq_expression, stack),
                    "!=" => operator2(neq_expression, stack),
                    "<" => operator2(le_expression, stack),
                    "<=" => operator2(leq_expression, stack),
                    ">" => operator2(ge_expression, stack),
                    ">=" => operator2(geq_expression, stack),

                    // macros and variables
                    _ => match macros.get(current) {
                        Some(f) => {
                            Expression::inject(f, stack, macros);
                        }
                        None => {
                            stack.push(Expression::Variable(current.to_string()));
                        }
                    },
                }
            }
        }
    }

    fn show_group(&self) -> String {
        match self {
            Expression::Magnitude(..) => self.show(),
            Expression::Variable(..) => self.show(),
            Expression::Unit(..) => self.show(),
            Expression::Sqrt(..) => self.show(),
            Expression::Rand(..) => self.show(),
            Expression::Not(..) => self.show(),
            _ => format!("( {} )", self.show()),
        }
    }

    pub fn show(&self) -> String {
        match self {
            Expression::Magnitude(magnitude) => format!("{}", magnitude),
            Expression::Variable(string) => string.to_string(),
            Expression::Add(items) => {
                let mut output = String::new();
                let mut iterator = items.iter();
                let first = iterator.next().unwrap();
                output.push_str(&first.show_group());
                for item in iterator {
                    if let Expression::Neg(_) = item {
                        output.push_str(&item.show_group());
                    } else {
                        output.push_str(" + ");
                        output.push_str(&item.show_group());
                    }
                }
                output
            }
            Expression::Neg(expr) => format!("-{}", expr.show_group()),
            Expression::Prod(items) => show_n_ary(" \\cdot ", items),
            Expression::Div(items) => {
                format!(" \\frac{{{}}}{{{}}} ", items[0].show(), items[1].show())
            }
            Expression::Unit(value, _name) => value.show(),
            Expression::Sqrt(expr) => format!("\\sqrt{{{}}}", expr.show()),
            Expression::Rand(items) => format!(
                "\\operatorname{{rand}}({}, {})",
                items[0].show_group(),
                items[1].show_group()
            ),
            Expression::And(items) => show_n_ary(" && ", items),
            Expression::Or(items) => show_n_ary(" || ", items),
            Expression::Not(expr) => format!("\\operatorname{{not}}({})", expr.show()),
            Expression::Eq(items) => show_n_ary(" == ", items),
            Expression::Neq(items) => {
                format!("{} \\not= {}", items[0].show_group(), items[1].show_group())
            }
            Expression::Le(items) => show_n_ary(" < ", items),
            Expression::Leq(items) => show_n_ary(" \\leq ", items),
            Expression::Ge(items) => show_n_ary(" > ", items),
            Expression::Geq(items) => show_n_ary(" \\geq ", items),
        }
    }

    pub fn value(&self, dict: &DictVariables) -> Magnitude {
        match self {
            Expression::Magnitude(mag) => mag.clone(),
            Expression::Variable(name) => {
                if let Some(expr) = dict.get(name) {
                    expr.value(dict)
                } else {
                    panic!("Variable {} not in dictionary", name)
                }
            }
            Expression::Add(operands) => value_n_ary(
                Magnitude::new(0.0, String::from("¿?")),
                |a, b| {
                    let unit = a.compatible_unit(&b).expect(&format!(
                        "Wrong units adding. Current_result: {:?}, next operand: {:?}\n",
                        a, b
                    ));
                    let value = a.value + b.value;
                    Magnitude { value, unit }
                },
                operands,
                dict,
            ),

            Expression::Neg(expr) => {
                let mag = expr.value(dict);
                Magnitude::new(-1.0 * mag.value, mag.unit)
            }
            Expression::Prod(operands) => value_n_ary(
                Magnitude::new(1.0, String::from("¿?")),
                |a, b| Magnitude {
                    value: a.value * b.value,
                    unit: String::from("¿?"),
                },
                operands,
                dict,
            ),
            Expression::Div(operands) => {
                let num = operands[0].value(dict);
                let den = operands[1].value(dict);

                Magnitude::new(num.value / den.value, String::from("¿?"))
            }
            Expression::Unit(expr, new_unit) => {
                let mut mag = expr.value(dict);
                mag.unit = mag
                    .compatible_unit_str(&new_unit)
                    .expect(&format!("Expression {:?} hasn't unit {}", expr, new_unit));

                mag
            }
            Expression::Sqrt(expr) => {
                let mag = expr.value(dict);
                Magnitude::new(mag.value.sqrt(), String::from("¿?"))
            }
            Expression::Rand(items) => {
                let min = items[0].value(dict).value;
                let max = items[1].value(dict).value;
                let unit = items[0]
                    .value(dict)
                    .compatible_unit(&items[1].value(dict))
                    .expect("Randon value with limits with different units");

                Magnitude {
                    value: (max - min) * random::<f64>() + min,
                    unit,
                }
            }
            Expression::And(operands) => value_n_ary(
                magnitude::TRUE.clone(),
                |a, b| {
                    if a == *magnitude::TRUE {
                        b.clone()
                    } else {
                        a.clone()
                    }
                },
                operands,
                dict,
            ),
            Expression::Or(operands) => value_n_ary(
                magnitude::FALSE.clone(),
                |a, b| {
                    if a != *magnitude::TRUE {
                        b.clone()
                    } else {
                        a.clone()
                    }
                },
                operands,
                dict,
            ),
            Expression::Not(expr) => {
                let value = expr.value(dict);

                if value == *magnitude::TRUE {
                    magnitude::FALSE.clone()
                } else {
                    magnitude::TRUE.clone()
                }
            }
            Expression::Eq(operands) => relation_n_ary(|a, b| a == b, operands, dict),
            Expression::Neq(operands) => relation_n_ary(|a, b| a != b, operands, dict),
            Expression::Le(operands) => relation_n_ary(|a, b| a < b, operands, dict),
            Expression::Leq(operands) => relation_n_ary(|a, b| a <= b, operands, dict),
            Expression::Ge(operands) => relation_n_ary(|a, b| a > b, operands, dict),
            Expression::Geq(operands) => relation_n_ary(|a, b| a >= b, operands, dict),
        }
    }
}

//------------------------------------------------

fn show_n_ary(sep: &str, items: &Arguments) -> String {
    items
        .iter()
        .map(Expression::show_group)
        .collect::<Vec<String>>()
        .join(sep)
}

//------------------------------------------------

fn value_n_ary(
    start: Magnitude,
    operand: fn(Magnitude, Magnitude) -> Magnitude,
    operands: &Arguments,
    dict: &DictVariables,
) -> Magnitude {
    let mut result = start;

    for op in operands {
        let next = op.value(dict);
        result = operand(result, next);
    }

    result
}

fn relation_n_ary(
    operand: fn(&Magnitude, &Magnitude) -> bool,
    operands: &Arguments,
    dict: &DictVariables,
) -> Magnitude {
    let mut iterator = operands.iter();
    let previous = iterator.next().unwrap();
    let mut previous = previous.value(dict);
    for it in iterator {
        let it = it.value(dict);
        it.compatible_unit(&previous).expect(&format!(
            "Wrong Units trying to compare {:?} and {:?}",
            previous, it
        ));

        if !(operand(&previous, &it)) {
            return magnitude::FALSE.clone();
        }
        previous = it;
    }
    magnitude::TRUE.clone()
}

//------------------------------------------------
//Operations over stack

fn pop(stack: &mut Stack) -> Expression {
    stack.pop().expect("Empty stack")
}

fn operator1(f: fn(Expression) -> Expression, stack: &mut Stack) {
    let op1 = pop(stack);

    stack.push(f(op1));
}

fn operator2(f: fn(Expression, Expression) -> Expression, stack: &mut Stack) {
    let op2 = pop(stack);
    let op1 = pop(stack);

    stack.push(f(op1, op2));
}

fn insert_magnitude(magnitude: Magnitude, stack: &mut Stack) {
    if magnitude.value >= 0.0 {
        stack.push(Expression::Magnitude(magnitude));
    } else {
        let mag_abs = Magnitude {
            value: magnitude.value.abs(),
            unit: magnitude.unit,
        };
        stack.push(Expression::Neg(Box::new(Expression::Magnitude(mag_abs))));
    }
}

//------------------------------------------------
//Operations over stack + dict

fn to_dict(stack: &mut Stack, dict: &mut DictVariables) {
    let variable = pop(stack); // variable name
    let content = pop(stack);

    match variable {
        Expression::Variable(name) => {
            dict.insert(name, content);
        }
        _ => {
            panic!("Inserting into dict without variable: {:?}", variable);
        }
    }
}

fn from_dict(stack: &mut Stack, dict: &DictVariables) {
    let variable = pop(stack); // variable name

    match variable {
        Expression::Variable(name) => match dict.get(&name) {
            Some(v) => {
                stack.push(v.clone());
            }
            None => {
                panic!("Variable: {} is not in dictionary", name);
            }
        },
        _ => {
            panic!("Getting from dict without variable: {:?}", variable);
        }
    }
}

//------------------------------------------------
// Operations with units

//------------------------------------------------
// Easy operations over expressions

fn units(value: Expression, unit: Expression) -> Expression {
    let unit = match unit {
        Expression::Variable(name) => name,
        _ => panic!("Impossible to use assign unit"),
    };

    Expression::Unit(Box::new(value), unit)
}

fn nounits(value: Expression) -> Expression {
    Expression::Unit(Box::new(value), String::new())
}

fn sqrt_expression(value: Expression) -> Expression {
    Expression::Sqrt(Box::new(value))
}

fn rand_expression(op1: Expression, op2: Expression) -> Expression {
    Expression::Rand(vec![op1, op2])
}

fn neq_expression(op1: Expression, op2: Expression) -> Expression {
    Expression::Neq(vec![op1, op2])
}

//------------------------------------------------
// Complex operations over expressions

fn add_expression(op1: Expression, op2: Expression) -> Expression {
    // factors extraction
    let mut operands1 = match op1 {
        Expression::Add(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::Add(operands) => operands,
        _ => vec![op2],
    };

    // operation
    operands1.append(&mut operands2);

    if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::Add(operands1)
    }
}

fn neg_expression(op1: Expression) -> Expression {
    match op1 {
        Expression::Add(summands) => {
            Expression::Add(summands.into_iter().map(neg_expression).collect())
        }
        Expression::Neg(expr) => *expr,
        _ => Expression::Neg(Box::new(op1)),
    }
}

fn prod_expression(op1: Expression, op2: Expression) -> Expression {
    let mut op1: Expression = op1;
    let mut op2: Expression = op2;

    // neg process
    let mut neg: bool = false;

    if let Expression::Neg(expr) = op1 {
        neg = !neg;
        op1 = *expr;
    }

    if let Expression::Neg(expr) = op2 {
        neg = !neg;
        op2 = *expr;
    }

    // operands extraction
    let mut operands1 = match op1 {
        Expression::Prod(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::Prod(operands) => operands,
        _ => vec![op2],
    };

    // product
    operands1.append(&mut operands2);

    let expr_without_sign = if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::Prod(operands1)
    };

    // ending with sign
    if !neg {
        expr_without_sign
    } else {
        Expression::Neg(Box::new(expr_without_sign))
    }
}

fn div_expression(op1: Expression, op2: Expression) -> Expression {
    let mut op1: Expression = op1;
    let mut op2: Expression = op2;

    // neg process
    let mut neg: bool = false;

    if let Expression::Neg(expr) = op1 {
        neg = !neg;
        op1 = *expr;
    }

    if let Expression::Neg(expr) = op2 {
        neg = !neg;
        op2 = *expr;
    }

    // factors extraction
    let num1: Expression;
    let mut den1: Expression = Expression::Prod(vec![]);
    let num2: Expression;
    let mut den2: Expression = Expression::Prod(vec![]);

    if let Expression::Div(factors) = op1 {
        num1 = factors[0].clone();
        den1 = factors[1].clone();
    } else {
        num1 = op1;
    }

    if let Expression::Div(factors) = op2 {
        num2 = factors[0].clone();
        den2 = factors[1].clone();
    } else {
        num2 = op2;
    }

    // product
    let num = prod_expression(num1, den2);
    let den = prod_expression(den1, num2);

    // ending with sign
    if !neg {
        Expression::Div(vec![num, den])
    } else {
        Expression::Neg(Box::new(Expression::Div(vec![num, den])))
    }
}

fn and_expression(op1: Expression, op2: Expression) -> Expression {
    // factors extraction
    let mut operands1 = match op1 {
        Expression::And(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::And(operands) => operands,
        _ => vec![op2],
    };

    // operation
    operands1.append(&mut operands2);

    if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::And(operands1)
    }
}

fn or_expression(op1: Expression, op2: Expression) -> Expression {
    // factors extraction
    let mut operands1 = match op1 {
        Expression::Or(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::Or(operands) => operands,
        _ => vec![op2],
    };

    // operation
    operands1.append(&mut operands2);

    if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::Or(operands1)
    }
}

fn not_expression(op1: Expression) -> Expression {
    match op1 {
        Expression::Not(expr) => *expr,
        _ => Expression::Not(Box::new(op1)),
    }
}

fn eq_expression(op1: Expression, op2: Expression) -> Expression {
    // factors extraction
    let mut operands1 = match op1 {
        Expression::Eq(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::Eq(operands) => operands,
        _ => vec![op2],
    };

    // operation
    operands1.append(&mut operands2);

    if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::Eq(operands1)
    }
}

fn le_expression(op1: Expression, op2: Expression) -> Expression {
    // factors extraction
    let mut operands1 = match op1 {
        Expression::Le(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::Le(operands) => operands,
        _ => vec![op2],
    };

    // operation
    operands1.append(&mut operands2);

    if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::Le(operands1)
    }
}

fn leq_expression(op1: Expression, op2: Expression) -> Expression {
    // factors extraction
    let mut operands1 = match op1 {
        Expression::Leq(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::Leq(operands) => operands,
        _ => vec![op2],
    };

    // operation
    operands1.append(&mut operands2);

    if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::Leq(operands1)
    }
}

fn ge_expression(op1: Expression, op2: Expression) -> Expression {
    // factors extraction
    let mut operands1 = match op1 {
        Expression::Ge(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::Ge(operands) => operands,
        _ => vec![op2],
    };

    // operation
    operands1.append(&mut operands2);

    if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::Ge(operands1)
    }
}

fn geq_expression(op1: Expression, op2: Expression) -> Expression {
    // factors extraction
    let mut operands1 = match op1 {
        Expression::Geq(operands) => operands,
        _ => vec![op1],
    };

    let mut operands2 = match op2 {
        Expression::Geq(operands) => operands,
        _ => vec![op2],
    };

    // operation
    operands1.append(&mut operands2);

    if operands1.len() == 1 {
        operands1[0].clone()
    } else {
        Expression::Geq(operands1)
    }
}
