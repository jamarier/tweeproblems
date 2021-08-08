// Expressions
//

//use lazy_static::lazy_static;
//use regex::Regex;
use std::collections::HashMap;

use crate::formulas;
use crate::magnitude::Magnitude;
//use crate::formulas::Algebraic;

pub type DictVariables = HashMap<String, Expression>;

// String values are future improvements ¿except variable?

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Magnitude(Magnitude),
    Variable(String),
    Bind(String, Box<Expression>),
    Add(Vec<Expression>),
    Neg(Box<Expression>),
    Prod(Vec<Expression>), //formula(String),
                           //algebra(String)
}

impl Expression {
    pub fn from(string: &str) -> Self {
        let items = string.split(" ");
        let mut stack: Vec<Expression> = vec![];

        for current in items {
            println!("current: {:?}", current);

            if let Some(value) = Magnitude::get(current) {
                stack.push(Expression::Magnitude(value));
            } else if &current[0..1] == ":" {
                let value = match stack.pop() {
                    Some(v) => Box::new(v),
                    None => panic!("Empty stack"),
                };
                let name = &current[1..];
                stack.push(Expression::Bind(name.to_string(), value));
            } else {
                match current {
                    "+" => operator2(add_expression, &mut stack),
                    "neg" => operator1(neg_expression, &mut stack),
                    "-" => {
                        operator1(neg_expression, &mut stack);
                        operator2(add_expression, &mut stack);
                    }
                    "*" => operator2(prod_expression, &mut stack),
                    _ => {
                        stack.push(Expression::Variable(current.to_string()));
                    }
                }
            }

            println!("stack: {:?}", stack);
        }

        if stack.len() == 1 {
            stack[0].clone()
        } else {
            panic!("Wrong Expression parsed: stack {:?}", stack)
        }
    }
}

//------------------------------------------------

fn pop(stack: &mut Vec<Expression>) -> Expression {
    match stack.pop() {
        Some(v) => v,
        None => panic!("Empty stack"),
    }
}

fn operator1(f: fn(Expression) -> Expression, stack: &mut Vec<Expression>) {
    let op1 = pop(stack);

    println!("op1 {:?}", op1);

    stack.push(f(op1));
}

fn operator2(f: fn(Expression, Expression) -> Expression, stack: &mut Vec<Expression>) {
    let op2 = pop(stack);
    let op1 = pop(stack);

    println!("op1 {:?} op2 {:?}", op1, op2);

    stack.push(f(op1, op2));
}

//------------------------------------------------

fn add_expression(op1: Expression, op2: Expression) -> Expression {
    match op1 {
        Expression::Add(mut summands1) => match op2 {
            Expression::Add(mut summands2) => {
                summands1.append(&mut summands2);
                Expression::Add(summands1)
            }
            _ => {
                summands1.push(op2);
                Expression::Add(summands1)
            }
        },
        _ => match op2 {
            Expression::Add(mut summands2) => {
                summands2.insert(0, op1);
                Expression::Add(summands2)
            }
            _ => Expression::Add(vec![op1, op2]),
        },
    }
}

fn neg_expression(op1: Expression) -> Expression {
    match op1 {
        Expression::Bind(string, expr) => Expression::Bind(string, Box::new(neg_expression(*expr))),
        Expression::Add(summands) => Expression::Add(
            summands
                .into_iter()
                .map(|expr| neg_expression(expr))
                .collect(),
        ),
        Expression::Neg(expr) => *expr,
        _ => Expression::Neg(Box::new(op1)),
    }
}

fn prod_expression(op1: Expression, op2: Expression) -> Expression {
    match op1 {
        Expression::Neg(expr1) => neg_expression(prod_expression(*expr1,op2)),
        Expression::Prod(mut factors1) => match op2 {
            Expression::Neg(expr2) => neg_expression(prod_expression(Expression::Prod(factors1), *expr2)),
            Expression::Prod(mut factors2) => {
                factors1.append(&mut factors2);
                Expression::Prod(factors1.to_vec())
            }
            _ => {
                factors1.push(op2);
                Expression::Prod(factors1.to_vec())
            },
        },
        _ => match op2 {
            Expression::Neg(expr2) => neg_expression(prod_expression(op1,*expr2)),
            Expression::Prod(mut factors2) => {
                factors2.insert(0, op1);
                Expression::Prod(factors2)
            }
            _ => Expression::Prod(vec![op1,op2])
        },
    }
}
