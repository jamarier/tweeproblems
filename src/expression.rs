// Expressions
//

use maplit::hashmap;
use rand::random;
use std::collections::HashMap;

use crate::macros::Macros;
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
    Sqrt(Box<Expression>),
    Rand(Vec<Expression>),
}

impl Expression {
    pub fn from(string: &str, macros: &Macros) -> Self {
        let mut stack: Vec<Expression> = vec![];

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

    fn inject(string: &str, stack: &mut Vec<Expression>, macros: &Macros) {
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
                        println!("END DEBUG\n*****\n");
                    }
                    "." => {
                        println!("*****\nTRACE");
                        println!(" String: {:?}", string);
                        println!(" Point {:?}", pop(stack).show());
                        println!("END TRACE\n*****\n");
                    }
                    "drop" => drop(stack),
                    "dup" => dup(stack),
                    "over" => over(stack),
                    "swap" => swap(stack),
                    "nip" => nip(stack),

                    // register operators
                    "!" => to_dict(stack, &mut dictionary),
                    "@" => from_dict(stack, &dictionary),

                    // arithmetic operators
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
                    "sqrt" => operator1(sqrt_expression, stack),
                    "rand" => operator2(rand_expression, stack),

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
            Expression::Sqrt(expr) => format!("\\sqrt{{{}}}", expr.show()),
            Expression::Rand(items) => {
                format!("rand({}, {})", items[0].show(), items[1].show())
            }
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
            Expression::Add(operands) => {
                let mut result = Magnitude::new(0.0, String::from("¿?"));

                for op in operands {
                    let mag = op.value(dict);
                    let unit = if let Some(s) = compatible_unit(&result, &mag) {
                        s
                    } else {
                        panic!("Wrong units adding {:?}.\n  Current result: {:?},\n  next operand:   {:?}\n",operands, result, mag);
                    };

                    result = Magnitude::new(result.value + mag.value, unit)
                }

                result
            }
            Expression::Neg(expr) => {
                let mag = expr.value(dict);
                Magnitude::new(-1.0 * mag.value, mag.unit)
            }
            Expression::Prod(operands) => {
                let mut result = Magnitude::new(1.0, String::from("¿?"));

                for op in operands {
                    let mag = op.value(dict);

                    result.value *= mag.value;
                }

                result
            }
            Expression::Div(operands) => {
                let num = operands[0].value(dict);
                let den = operands[1].value(dict);

                Magnitude::new(num.value / den.value, String::from("¿?"))
            }
            Expression::Unit(new_unit, expr) => {
                let mut mag = expr.value(dict);
                mag.unit = if let Some(s) = compatible_unit(
                    &mag,
                    &Magnitude {
                        value: 0.0,
                        unit: new_unit.clone(),
                    },
                ) {
                    s
                } else {
                    panic!("Expression {:?} hasn't unit {}", expr, new_unit);
                };

                mag
            }
            Expression::Sqrt(expr) => {
                let mag = expr.value(dict);
                Magnitude::new(mag.value.sqrt(), String::from("¿?"))
            }
            Expression::Rand(items) => {
                let min = items[0].value(dict).value;
                let max = items[1].value(dict).value;
                let unit = compatible_unit(&items[0].value(dict), &items[1].value(dict))
                    .expect("Randon value with limits with different units");

                Magnitude {
                    value: (max - min) * random::<f64>() + min,
                    unit,
                }
            }
        }
    }
}

//------------------------------------------------

//------------------------------------------------
//Operations over stack

fn pop(stack: &mut Vec<Expression>) -> Expression {
    /*
    match stack.pop() {
        Some(v) => v,
        None => panic!("Empty stack"),
    }
    */

    stack.pop().expect("Empty stack")
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

fn drop(stack: &mut Vec<Expression>) {
    stack.pop();
}

fn dup(stack: &mut Vec<Expression>) {
    let n1 = pop(stack);

    stack.push(n1.clone());
    stack.push(n1);
}

fn over(stack: &mut Vec<Expression>) {
    let n2 = pop(stack);
    let n1 = pop(stack);

    stack.push(n1.clone());
    stack.push(n2);
    stack.push(n1);
}

fn swap(stack: &mut Vec<Expression>) {
    let n2 = pop(stack);
    let n1 = pop(stack);

    stack.push(n2);
    stack.push(n1);
}

fn nip(stack: &mut Vec<Expression>) {
    let n2 = pop(stack);
    let _n1 = pop(stack);

    stack.push(n2);
}

fn insert_magnitude(magnitude: Magnitude, stack: &mut Vec<Expression>) {
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
// Operations with units

fn compatible_unit(a: &Magnitude, b: &Magnitude) -> Option<String> {
    if a.unit == "¿?" {
        return Some(b.unit.clone());
    } else if b.unit == "¿?" {
        return Some(a.unit.clone());
    } else if a.unit == b.unit {
        return Some(a.unit.clone());
    } else {
        return None;
    }
}

//------------------------------------------------
// Easy operations over expressions

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

fn sqrt_expression(value: Expression) -> Expression {
    Expression::Sqrt(Box::new(value))
}

fn rand_expression(op1: Expression, op2: Expression) -> Expression {
    Expression::Rand(vec![op1, op2])
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
