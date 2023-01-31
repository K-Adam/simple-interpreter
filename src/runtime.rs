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
            span: *span,
        });
    };

    println!("Input: ");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| RuntimeError {
            message: format!("Error when reading from console: {err:?}"),
            span: *span,
        })?;

    input.trim().parse::<i32>().map_err(|err| RuntimeError {
        message: format!("Error when converting string to integer: {input}, {err:?}"),
        span: *span,
    })
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
        n => Err(RuntimeError {
            message: format!("Too many arguments for print. Expected 0 or 1, got {n}"),
            span: *span,
        }),
    }
}
