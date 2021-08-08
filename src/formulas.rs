// Structures to support the use of formulas in histories

//use lazy_static::lazy_static;
//use maplit::hashmap;
//use std::collections::HashMap;

//use crate::expression::Expression;

/*pub trait Algebraic {
    fn build(stack: &mut Vec<Expression>);
}
*/

/*
pub struct Prod {
}

impl Algebraic for Prod {
    fn build(stack: &mut Vec<Expression>) {
        let op2 = match stack.pop() {
            Some(v) => v,
            None => panic!("Empty stack")
        };
        let op1 = match stack.pop() {
            Some(v) => v,
            None => panic!("Empty stack")
        };

        println!("op1 {:?} op2 {:?}",op1,op2);

        if let Expression::Prod(mut sumands1) = op1 {
            if let Expression::Prod(mut sumands2) = op2 {
                sumands1.append(&mut sumands2);
            } else {
                sumands1.push(op2);
            }
            stack.push(Expression::Prod(sumands1));
        } else {
            if let Expression::Prod(mut sumands2) = op2 {
                sumands2.insert(0,op1);
                stack.push(Expression::Prod(sumands2));
            } else {
                stack.push(Expression::Prod(vec![op1,op2]));
            }
        }
    }
}
*/

/*
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
}
*/
