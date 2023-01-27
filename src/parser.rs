use crate::{
    lexer::{operator_precedence, Operator, Token, TokenNode, Tokenizer},
    utils::{Span, SpanError},
};

pub struct Parser<T: Tokenizer> {
    tokenizer: T,
}

pub type ParserError = SpanError;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AstNode<T> {
    pub node: T,
    pub span: Span,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Number(i32),
    BinaryOperator(Box<AstNode<Expression>>, Operator, Box<AstNode<Expression>>),
    Identifier(String),
    Call(AstNode<FunctionCall>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Program {
    pub lines: Vec<AstNode<Line>>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Line {
    Assignment(String, AstNode<Expression>),
    Reassignment(String, AstNode<Expression>),
    Call(AstNode<FunctionCall>),
    Loop(AstNode<Expression>, Vec<AstNode<Line>>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<AstNode<Expression>>,
}

macro_rules! take_token {
    ($tokenizer:expr, $pattern:pat) => {
        $tokenizer.next().and_then(|token_node| match token_node {
            TokenNode {
                token: $pattern,
                span,
            } => Ok(span),
            TokenNode { token, span } => Err(ParserError {
                message: format!(
                    "Unexpected token {:?}, expected: {}",
                    token,
                    stringify!($pattern)
                ),
                span,
            }),
        })
    };
}

impl<T: Tokenizer> Parser<T> {
    pub fn new(tokenizer: T) -> Parser<T> {
        Parser { tokenizer }
    }

    pub fn parse(&mut self) -> Result<AstNode<Program>, ParserError> {
        let mut span = self.tokenizer.get_empty_span()?;
        let mut lines = Vec::new();

        while self.tokenizer.peek()? != Token::Eof {
            let node = self.parse_line()?;
            span.end = node.span.end;
            lines.push(node);
        }

        take_token!(self.tokenizer, Token::Eof)?;

        Ok(AstNode {
            node: Program { lines },
            span,
        })
    }

    fn parse_line(&mut self) -> Result<AstNode<Line>, ParserError> {
        match self.tokenizer.peek()?.token {
            Token::Var => self.parse_assignment(),
            Token::While => self.parse_loop(),
            Token::Identifier(_) => self.parse_reassignment_or_call(),
            other => Err(ParserError {
                message: format!("Unexpected token {other:?}, expected: Var, While, Identifier"),
                span: self.tokenizer.peek()?.span,
            }),
        }
    }

    fn parse_assignment(&mut self) -> Result<AstNode<Line>, ParserError> {
        let var_span = take_token!(self.tokenizer, Token::Var)?;

        let identifier = match self.tokenizer.next()? {
            TokenNode {
                token: Token::Identifier(name),
                span: _,
            } => name,
            TokenNode { token, span } => {
                return Err(ParserError {
                    message: format!("Unexpected token {token:?}, expected: Identifier"),
                    span,
                })
            }
        };

        take_token!(self.tokenizer, Token::Equals)?;

        let expression = self.parse_expression()?;

        let semicolon_span = take_token!(self.tokenizer, Token::SemiColon)?;

        Ok(AstNode {
            node: Line::Assignment(identifier, expression),
            span: Span {
                start: var_span.start,
                end: semicolon_span.end,
            },
        })
    }

    fn parse_loop(&mut self) -> Result<AstNode<Line>, ParserError> {
        let while_span = take_token!(self.tokenizer, Token::While)?;

        let condition = self.parse_expression()?;

        take_token!(self.tokenizer, Token::OpeningCurlyBracket)?;

        let mut lines = Vec::new();

        while self.tokenizer.peek()?.token != Token::ClosingCurlyBracket {
            lines.push(self.parse_line()?);
        }

        let close_span = take_token!(self.tokenizer, Token::ClosingCurlyBracket)?;

        Ok(AstNode {
            node: Line::Loop(condition, lines),
            span: Span {
                start: while_span.start,
                end: close_span.end,
            },
        })
    }

    fn parse_reassignment_or_call(&mut self) -> Result<AstNode<Line>, ParserError> {
        let (identifier, identifier_span) = match self.tokenizer.next()? {
            TokenNode {
                token: Token::Identifier(name),
                span,
            } => (name, span),
            TokenNode { token, span } => {
                return Err(ParserError {
                    message: format!("Unexpected token {token:?}, expected: Identifier"),
                    span,
                })
            }
        };

        if self.tokenizer.peek()? == Token::Equals {
            // Reassignment

            take_token!(self.tokenizer, Token::Equals)?;

            let expression = self.parse_expression()?;

            let semicolon_span = take_token!(self.tokenizer, Token::SemiColon)?;

            Ok(AstNode {
                node: Line::Reassignment(identifier, expression),
                span: Span {
                    start: identifier_span.start,
                    end: semicolon_span.end,
                },
            })
        } else {
            // Call

            take_token!(self.tokenizer, Token::OpeningParenthesis)?;

            let arguments = self.parse_arguments()?;

            let close_span = take_token!(self.tokenizer, Token::ClosingParenthesis)?;

            let semicolon_span = take_token!(self.tokenizer, Token::SemiColon)?;

            let call = AstNode {
                node: FunctionCall {
                    name: identifier,
                    arguments,
                },
                span: Span {
                    start: identifier_span.start,
                    end: close_span.end,
                },
            };

            Ok(AstNode {
                node: Line::Call(call),
                span: Span {
                    start: identifier_span.start,
                    end: semicolon_span.end,
                },
            })
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<AstNode<Expression>>, ParserError> {
        let mut arguments = Vec::new();

        while self.tokenizer.peek()? != Token::ClosingParenthesis {
            arguments.push(self.parse_expression()?);

            if self.tokenizer.peek()? != Token::ClosingParenthesis {
                take_token!(self.tokenizer, Token::Comma)?;
            }
        }

        Ok(arguments)
    }

    // Expression without operators
    fn parse_simple_expression(&mut self) -> Result<AstNode<Expression>, ParserError> {
        match self.tokenizer.next()? {
            TokenNode {
                token: Token::Number(number),
                span,
            } => Ok(AstNode {
                node: Expression::Number(number),
                span,
            }),
            TokenNode {
                token: Token::OpeningParenthesis,
                span: _,
            } => {
                let expression = self.parse_expression()?;

                take_token!(self.tokenizer, Token::ClosingParenthesis)?;

                Ok(expression)
            }
            TokenNode {
                token: Token::Identifier(name),
                span,
            } => {
                if self.tokenizer.peek()? == Token::OpeningParenthesis {
                    take_token!(self.tokenizer, Token::OpeningParenthesis)?;

                    let arguments = self.parse_arguments()?;

                    let close_span = take_token!(self.tokenizer, Token::ClosingParenthesis)?;

                    Ok(AstNode {
                        node: Expression::Call(AstNode {
                            node: FunctionCall { name, arguments },
                            span: Span {
                                start: span.start,
                                end: close_span.end,
                            },
                        }),
                        span,
                    })
                } else {
                    Ok(AstNode {
                        node: Expression::Identifier(name),
                        span,
                    })
                }
            }
            TokenNode { token, span } => Err(ParserError {
                message: format!(
                    "Unexpected token {token:?}, expected number, opening parenthesis, identifier"
                ),
                span,
            }),
        }
    }

    fn parse_expression(&mut self) -> Result<AstNode<Expression>, ParserError> {
        self.parse_operator_expression(0)
    }

    fn parse_operator_expression(
        &mut self,
        precedence: u8,
    ) -> Result<AstNode<Expression>, ParserError> {
        let mut left = self.parse_simple_expression()?;

        while let Token::Operator(operator) = self.tokenizer.peek()?.token {
            let op = operator.clone();
            let next_precedence = operator_precedence(&op);
            if next_precedence < precedence {
                break;
            }

            take_token!(self.tokenizer, Token::Operator(_))?;
            let right = self.parse_operator_expression(next_precedence)?;

            let result_span = Span {
                start: left.span.start,
                end: right.span.end,
            };
            left = AstNode {
                node: Expression::BinaryOperator(Box::new(left), op, Box::new(right)),
                span: result_span,
            };
        }

        Ok(left)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{
        lexer::{Operator, Token, TokenNode, TokenResult, Tokenizer, TokenizerError},
        parser::{AstNode, Expression, Parser},
        utils::Span,
    };

    struct MockTokenizer {
        tokens: VecDeque<Token>,
    }

    impl Tokenizer for MockTokenizer {
        fn next(&mut self) -> TokenResult {
            self.tokens
                .pop_front()
                .map(|token| TokenNode {
                    token,
                    span: Span { start: 0, end: 0 },
                })
                .ok_or(TokenizerError {
                    message: "Empty".into(),
                    span: Span { start: 0, end: 0 },
                })
        }

        fn peek(&mut self) -> TokenResult {
            let token = self
                .tokens
                .front()
                .ok_or(TokenizerError {
                    message: "Empty".into(),
                    span: Span { start: 0, end: 0 },
                })?
                .clone();

            Ok(TokenNode {
                token,
                span: Span { start: 0, end: 0 },
            })
        }

        fn get_empty_span(&mut self) -> Result<Span, TokenizerError> {
            Ok(Span { start: 0, end: 0 })
        }

        fn collect_tokens(&mut self) -> Result<Vec<Token>, TokenizerError> {
            let mut result = Vec::new();

            while let Some(token) = self.tokens.pop_front() {
                result.push(token);
            }

            Ok(result)
        }
    }

    fn ast<N>(node: N) -> AstNode<N> {
        AstNode {
            node,
            span: Span { start: 0, end: 0 },
        }
    }

    fn tokenizer<const C: usize>(tokens: [Token; C]) -> MockTokenizer {
        MockTokenizer {
            tokens: VecDeque::from(tokens),
        }
    }

    #[test]
    fn parse_operator_expression_simple() {
        let tokenizer = tokenizer([
            Token::Number(1),
            Token::Operator(Operator::Plus),
            Token::Number(2),
            Token::Eof,
        ]);
        let mut parser = Parser { tokenizer };
        let exp = parser.parse_expression().unwrap();
        let expected = ast(Expression::BinaryOperator(
            Box::new(ast(Expression::Number(1))),
            Operator::Plus,
            Box::new(ast(Expression::Number(2))),
        ));

        assert_eq!(exp, expected);
    }

    #[test]
    fn parse_operator_expression_precedence_left() {
        let tokenizer = tokenizer([
            Token::Number(1),
            Token::Operator(Operator::Multiplication),
            Token::Number(2),
            Token::Operator(Operator::Plus),
            Token::Number(3),
            Token::Eof,
        ]);
        let mut parser = Parser { tokenizer };
        let exp = parser.parse_expression().unwrap();
        let expected = ast(Expression::BinaryOperator(
            Box::new(ast(Expression::BinaryOperator(
                Box::new(ast(Expression::Number(1))),
                Operator::Multiplication,
                Box::new(ast(Expression::Number(2))),
            ))),
            Operator::Plus,
            Box::new(ast(Expression::Number(3))),
        ));

        assert_eq!(exp, expected);
    }

    #[test]
    fn parse_operator_expression_precedence_right() {
        let tokenizer = tokenizer([
            Token::Number(1),
            Token::Operator(Operator::Plus),
            Token::Number(2),
            Token::Operator(Operator::Multiplication),
            Token::Number(3),
            Token::Eof,
        ]);
        let mut parser = Parser { tokenizer };
        let exp = parser.parse_expression().unwrap();
        let expected = ast(Expression::BinaryOperator(
            Box::new(ast(Expression::Number(1))),
            Operator::Plus,
            Box::new(ast(Expression::BinaryOperator(
                Box::new(ast(Expression::Number(2))),
                Operator::Multiplication,
                Box::new(ast(Expression::Number(3))),
            ))),
        ));

        assert_eq!(exp, expected);
    }
}
