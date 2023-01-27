use std::collections::HashMap;

use crate::lexer::Operator;
use crate::runtime::{function_input, function_print};
use crate::utils::SpanError;

use crate::parser::{AstNode, Expression, FunctionCall, Line, Program};

pub type RuntimeError = SpanError;

type CustomFunction =
    fn(&Evaluator, &mut State, &AstNode<FunctionCall>) -> Result<i32, RuntimeError>;

pub struct State {
    pub variables: HashMap<String, i32>,
    pub functions: HashMap<String, CustomFunction>,
}

pub struct Evaluator {}

impl Evaluator {
    pub fn evaluate(
        &self,
        AstNode {
            node: program,
            span: _,
        }: AstNode<Program>,
    ) -> Result<(), RuntimeError> {
        let mut state = State {
            variables: HashMap::new(),
            functions: HashMap::from([
                ("input".into(), function_input as CustomFunction),
                ("print".into(), function_print as CustomFunction),
            ]),
        };
        for line in program.lines {
            self.evaluate_line(&mut state, &line)?
        }
        Ok(())
    }

    fn evaluate_line(
        &self,
        state: &mut State,
        AstNode { node: line, span }: &AstNode<Line>,
    ) -> Result<(), RuntimeError> {
        match line {
            Line::Assignment(name, expression) => {
                let value = self.evaluate_expression(state, expression)?;

                // use of unstable library feature 'map_try_insert'
                if state.variables.contains_key(name) {
                    return Err(RuntimeError {
                        message: format!("Variable {name} is already defined"),
                        span: span.clone(),
                    });
                };
                state.variables.insert(name.clone(), value);
                Ok(())
            }
            Line::Reassignment(name, expression) => {
                let value = self.evaluate_expression(state, expression)?;
                let var_ref = state.variables.get_mut(name).ok_or(RuntimeError {
                    message: format!("Variable {name} is not defined"),
                    span: span.clone(),
                })?;
                *var_ref = value;
                Ok(())
            }
            Line::Call(function_call) => self
                .evaluate_function_call(state, function_call)
                .map(|_| ()),
            Line::Loop(condition, lines) => {
                while self.evaluate_expression(state, condition)? != 0 {
                    for line in lines {
                        self.evaluate_line(state, line)?;
                    }
                }
                Ok(())
            }
        }
    }

    fn evaluate_function_call(
        &self,
        state: &mut State,
        ast_node: &AstNode<FunctionCall>,
    ) -> Result<i32, RuntimeError> {
        state
            .functions
            .get(&ast_node.node.name)
            .ok_or(RuntimeError {
                message: format!("Function {} not found", ast_node.node.name),
                span: ast_node.span.clone(),
            })?(self, state, ast_node)
    }

    pub fn evaluate_expression(
        &self,
        state: &mut State,
        AstNode {
            node: expression,
            span,
        }: &AstNode<Expression>,
    ) -> Result<i32, RuntimeError> {
        match expression {
            Expression::Number(value) => Ok(*value),
            Expression::Call(function_call) => self.evaluate_function_call(state, function_call),
            Expression::BinaryOperator(left, op, right) => {
                let left_value = self.evaluate_expression(state, left)?;
                let right_value = self.evaluate_expression(state, right)?;
                Ok(match op {
                    Operator::Plus => left_value + right_value,
                    Operator::Minus => left_value - right_value,
                    Operator::Multiplication => left_value * right_value,
                    Operator::LessThan => (left_value < right_value) as i32,
                })
            }
            Expression::Identifier(name) => match state.variables.get(name) {
                Some(value) => Ok(*value),
                None => Err(RuntimeError {
                    message: format!("Variable does not exist: {name}"),
                    span: span.clone(),
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Operator;
    use crate::parser::{AstNode, Expression};
    use crate::utils::Span;
    use std::collections::HashMap;

    use super::{Evaluator, State};

    macro_rules! ast {
        ($node:expr) => {
            AstNode {
                node: $node,
                span: Span { start: 0, end: 0 },
            }
        };
    }

    #[test]
    fn test_evaluate_expression() {
        let ast = ast!(Expression::BinaryOperator(
            Box::new(ast!(Expression::Number(1))),
            Operator::Plus,
            Box::new(ast!(Expression::Number(2))),
        ));
        let mut state = State {
            variables: HashMap::new(),
            functions: HashMap::new(),
        };
        let evaluator = Evaluator {};
        let result = evaluator.evaluate_expression(&mut state, &ast).unwrap();
        let expected = 3;

        assert_eq!(expected, result);
    }
}
