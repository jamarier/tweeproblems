// Expressions
//

//use lazy_static::lazy_static;
//use regex::Regex;
use maplit::hashmap;
use std::collections::HashMap;

use crate::formulas::FORMULAS;
use crate::magnitude::Magnitude;

pub type DictVariables = HashMap<String, Expression>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Magnitude(Magnitude),
    Variable(String),
    Add(Vec<Expression>),
    Neg(Box<Expression>),
    Prod(Vec<Expression>),
    Div(Vec<Expression>),
    Unit(String, Box<Expression>), // Assign Unit if unit = "", verify unit if previus expression have unit
}

impl Expression {
    pub fn from(string: &str) -> Self {
        let mut stack: Vec<Expression> = vec![];
        let mut dictionary: DictVariables = hashmap! {};

        Expression::inject(string, &mut stack, &mut dictionary)
    }

    fn inject(string: &str, stack: &mut Vec<Expression>, dictionary: &mut DictVariables) -> Self {
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
            if false && !current.is_empty() {
                println!("stack: {:?}", stack);
                println!("current: {:?}", current);
            }
            if current.is_empty() {
            } else if let Some(magnitude) = Magnitude::get(current) {
                if magnitude.value >= 0.0 {
                    stack.push(Expression::Magnitude(magnitude));
                } else {
                    let mag_abs = Magnitude {
                        value: magnitude.value.abs(),
                        unit: magnitude.unit,
                    };
                    stack.push(Expression::Neg(Box::new(Expression::Magnitude(mag_abs))));
                }
            } else {
                match current {
                    "!" => to_dict(stack, dictionary),
                    "@" => from_dict(stack, dictionary),
                    ":" => operator2(units, stack),
                    "::" => operator1(nounits, stack),
                    "+" => operator2(add_expression, stack),
                    "neg" => operator1(neg_expression, stack),
                    "-" => {
                        operator1(neg_expression, stack);
                        operator2(add_expression, stack);
                    }
                    "*" => operator2(prod_expression, stack),
                    "/" => operator2(div_expression, stack),
                    _ => match FORMULAS.get(current) {
                        Some(f) => {
                            Expression::inject(f, stack, dictionary);
                        }
                        None => {
                            stack.push(Expression::Variable(current.to_string()));
                        }
                    },
                }
            }
        }

        if stack.len() == 1 {
            stack[0].clone()
        } else {
            panic!("Wrong Expression parsed: stack {:?}", stack)
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
                output.push_str(&first.show());
                for item in iterator {
                    if let Expression::Neg(_) = item {
                        output.push_str(&item.show());
                    } else {
                        output.push_str(" + ");
                        output.push_str(&item.show());
                    }
                }
                output
            }
            Expression::Neg(expr) => format!(" -{}", expr.show()),
            Expression::Prod(items) => items
                .iter()
                .map(|it| match it {
                    Expression::Add(_) => format!("({})", it.show()),
                    _ => it.show(),
                })
                .collect::<Vec<String>>()
                .join(" \\cdot "),
            Expression::Div(items) => {
                format!(" \\frac{{{}}}{{{}}} ", items[0].show(), items[1].show())
            }
            Expression::Unit(_name, value) => value.show(),
        }
    }

    pub fn value(&self, global_dict: &DictVariables, local_dict: &DictVariables) -> Magnitude {
        match self {
            Expression::Magnitude(mag) => mag.clone(),
            Expression::Variable(name) => {
                if let Some(expr) = global_dict.get(name) {
                    expr.value(global_dict, local_dict)
                } else if let Some(expr) = local_dict.get(name) {
                    expr.value(global_dict, local_dict)
                } else {
                    panic!("Variable {} not in dictionaries", name)
                }
            }
            Expression::Add(operands) => {
                let mut result = Magnitude::new(0.0, String::from("¿?"));

                for op in operands {
                    let mag = op.value(global_dict, local_dict);
                    if !(result.unit == "¿?" || mag.unit == "¿?" || result.unit == mag.unit) {
                        panic!("Wrong units adding {:?}.\n  Current result: {:?},\n  next operand:   {:?}\n",operands, result, mag);
                    }

                    let value = result.value + mag.value;
                    let unit = if result.unit == "¿?" {
                        mag.unit
                    } else {
                        result.unit
                    };
                    result = Magnitude::new(value, unit)
                }

                result
            }
            Expression::Neg(expr) => {
                let mag = expr.value(global_dict, local_dict);
                Magnitude::new(-1.0 * mag.value, mag.unit)
            }
            Expression::Prod(operands) => {
                let mut result = Magnitude::new(1.0, String::from("¿?"));

                for op in operands {
                    let mag = op.value(global_dict, local_dict);

                    result.value *= mag.value;
                }

                result
            }
            Expression::Div(operands) => {
                let num = operands[0].value(global_dict, local_dict);
                let den = operands[1].value(global_dict, local_dict);

                Magnitude::new(num.value / den.value, String::from("¿?"))
            }
            Expression::Unit(unit, expr) => {
                let mut mag = expr.value(global_dict, local_dict);
                if &mag.unit != "¿?" && &mag.unit != unit {
                    panic!("Expression {:?} hasn't unit {}", expr, unit);
                }
                mag.unit = unit.to_owned();
                mag
            }
        }
    }
}

//------------------------------------------------

//------------------------------------------------
//Operations over stack

fn pop(stack: &mut Vec<Expression>) -> Expression {
    match stack.pop() {
        Some(v) => v,
        None => panic!("Empty stack"),
    }
}

fn operator1(f: fn(Expression) -> Expression, stack: &mut Vec<Expression>) {
    let op1 = pop(stack);

    stack.push(f(op1));
}

fn operator2(f: fn(Expression, Expression) -> Expression, stack: &mut Vec<Expression>) {
    let op2 = pop(stack);
    let op1 = pop(stack);

    stack.push(f(op1, op2));
}

//------------------------------------------------
//Operations over stack + dict

fn to_dict(stack: &mut Vec<Expression>, dict: &mut DictVariables) {
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

fn from_dict(stack: &mut Vec<Expression>, dict: &DictVariables) {
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
//Operations over expressions

fn units(value: Expression, unit: Expression) -> Expression {
    let unit = match unit {
        Expression::Variable(name) => name,
        _ => panic!("Impossible to use assign unit"),
    };

    Expression::Unit(unit, Box::new(value))
}

fn nounits(value: Expression) -> Expression {
    Expression::Unit(String::new(), Box::new(value))
}

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
