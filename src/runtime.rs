use std::io;

use crate::{
    evaluator::{Evaluator, RuntimeError, State},
    parser::{AstNode, Expression, FunctionCall},
};

pub fn function_input(
    _: &Evaluator,
    _: &mut State,
    AstNode {
        node: function_call,
        span,
    }: &AstNode<FunctionCall>,
) -> Result<i32, RuntimeError> {
    if !function_call.arguments.is_empty() {
        return Err(RuntimeError {
            message: "Input function does not take any arguments".into(),
            span: span.clone(),
        });
    };

    println!("Input: ");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => match input.trim().parse::<i32>() {
            Ok(num) => Ok(num),
            Err(err) => Err(RuntimeError {
                message: format!("Error when parsing: {input}, {err:?}"),
                span: span.clone(),
            }),
        },
        Err(err) => Err(RuntimeError {
            message: format!("Error when reading from console: {err:?}"),
            span: span.clone(),
        }),
    }
}

pub fn function_print(
    evaluator: &Evaluator,
    state: &mut State,
    AstNode {
        node: function_call,
        span,
    }: &AstNode<FunctionCall>,
) -> Result<i32, RuntimeError> {
    match function_call.arguments.len() {
        0 => {
            println!("{:?}", state.variables);
            Ok(0)
        }
        1 => {
            let expression = function_call.arguments.get(0).unwrap();
            let value = evaluator.evaluate_expression(state, expression)?;
            match expression.node {
                Expression::Identifier(ref name) => println!("{name} = {value:?}"),
                _ => println!("Result = {value:?}"),
            };
            Ok(0)
        }
        _ => Err(RuntimeError {
            message: "Too many arguments for print".into(),
            span: span.clone(),
        }),
    }
}
