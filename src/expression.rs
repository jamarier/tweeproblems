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
    Prod(Vec<Expression>),
    Div(Vec<Expression>),
    //formula(String),
                           //algebra(String)
}

impl Expression {
    pub fn from(string: &str) -> Self {
        let items = string.split(" ");
        let mut stack: Vec<Expression> = vec![];

        for current in items {
            println!("current: {:?}", current);

            if current == "" {
            } else if let Some(value) = Magnitude::get(current) {
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
                    "/" => operator2(div_expression, &mut stack),
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

    // factors extraction
    let mut operands1 = match op1 {
        Expression::Add(operands) => operands,
        _ => vec![op1]
    };

    let mut operands2 = match op2 {
        Expression::Add(operands) => operands,
        _ => vec![op2]
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
    let mut op1: Expression = op1;
    let mut op2: Expression = op2;

    // neg process
    let mut neg : bool = false;

    if let Expression::Neg(expr) = op1 {
        neg = !neg ;
        op1 = *expr;
    }

    if let Expression::Neg(expr) = op2 {
        neg = !neg ;
        op2 = *expr;
    }

    // operands extraction
    let mut operands1 = match op1 {
        Expression::Prod(operands) => operands,
        _ => vec![op1]
    };

    let mut operands2 = match op2 {
        Expression::Prod(operands) => operands,
        _ => vec![op2]
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
    let mut neg : bool = false;

    if let Expression::Neg(expr) = op1 {
        neg = !neg ;
        op1 = *expr;
    }

    if let Expression::Neg(expr) = op2 {
        neg = !neg ;
        op2 = *expr;
    }

    // factors extraction
    let mut num1: Expression;
    let mut den1: Expression = Expression::Prod(vec![]);
    let mut num2: Expression;
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
    let num = prod_expression(num1,den2);
    let den = prod_expression(den1,num2);

    // ending with sign
    if !neg {
        Expression::Div(vec![num,den])
    } else {
        Expression::Neg(Box::new(Expression::Div(vec![num,den])))
    }
}

