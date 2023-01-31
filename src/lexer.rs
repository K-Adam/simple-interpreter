use crate::utils::{Span, SpanError};
use lazy_regex::regex;
use regex::{Captures, Regex};
use std::str;
use substring::Substring;

pub type TokenizerError = SpanError;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiplication,
    LessThan,
}

pub fn operator_precedence(op: &Operator) -> u8 {
    match op {
        Operator::LessThan => 1,
        Operator::Plus | Operator::Minus => 2,
        Operator::Multiplication => 3,
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Token {
    OpeningParenthesis,
    ClosingParenthesis,
    OpeningCurlyBracket,
    ClosingCurlyBracket,
    SemiColon,
    Equals,
    Number(i32),
    Identifier(String),
    Operator(Operator),
    Var,
    While,
    Comma,
    Eof,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenNode {
    pub token: Token,
    pub span: Span,
}

impl TokenNode {
    pub fn new(token: Token, start: usize, end: usize) -> TokenNode {
        TokenNode {
            token,
            span: Span { start, end },
        }
    }
}

impl std::cmp::PartialEq<Token> for TokenNode {
    fn eq(&self, token: &Token) -> bool {
        self.token == *token
    }
}

pub type TokenResult = Result<TokenNode, TokenizerError>;

pub trait Tokenizer {
    fn next(&mut self) -> TokenResult;
    fn peek(&mut self) -> TokenResult;
    fn get_empty_span(&mut self) -> Result<Span, TokenizerError>;
    fn collect_tokens(&mut self) -> Result<Vec<Token>, TokenizerError>;
}

pub struct SimpleTokenizer<'a> {
    data: &'a str,
    cursor: usize,
    next: Option<TokenResult>,
    rules: Vec<TokenizerRule>,
    matches_keyword: Regex,
    terminated: bool,
}

pub enum TokenizerRule {
    Char(char, Token),
    String(&'static str, Token),
    Regex(Regex, fn(&Captures) -> Token),
}

impl SimpleTokenizer<'_> {
    pub fn new(data: &str) -> SimpleTokenizer {
        let rules = vec![
            TokenizerRule::Char('(', Token::OpeningParenthesis),
            TokenizerRule::Char(')', Token::ClosingParenthesis),
            TokenizerRule::Char('{', Token::OpeningCurlyBracket),
            TokenizerRule::Char('}', Token::ClosingCurlyBracket),
            TokenizerRule::Char(';', Token::SemiColon),
            TokenizerRule::Char('=', Token::Equals),
            TokenizerRule::Char('+', Token::Operator(Operator::Plus)),
            TokenizerRule::Char('-', Token::Operator(Operator::Minus)),
            TokenizerRule::Char('*', Token::Operator(Operator::Multiplication)),
            TokenizerRule::Char('<', Token::Operator(Operator::LessThan)),
            TokenizerRule::Char(',', Token::Comma),
            TokenizerRule::Regex(
                Regex::new(r"^([a-zA-Z][a-zA-Z0-9_]*)").unwrap(),
                |cap: &Captures| Token::Identifier(cap[0].to_string()),
            ),
            TokenizerRule::String("var", Token::Var),
            TokenizerRule::String("while", Token::While),
            TokenizerRule::Regex(Regex::new(r"^(\d+)").unwrap(), |cap: &Captures| {
                Token::Number(cap[0].parse().unwrap())
            }),
        ];

        // Do not match keywords as identifiers
        let matches_keyword = {
            let string_rules = rules
                .iter()
                .filter_map(|rule| match rule {
                    TokenizerRule::String(str, _) => Some(*str),
                    _ => None,
                })
                .collect::<Vec<&str>>()
                .join("|");
            let string_rules_re = format!("^({string_rules})$");

            Regex::new(string_rules_re.as_str()).unwrap()
        };

        SimpleTokenizer {
            data,
            cursor: 0,
            next: None,
            rules,
            matches_keyword,
            terminated: false,
        }
    }

    fn read(&self, start_index: usize) -> TokenResult {
        let view = self.data.substring(start_index, self.data.len());

        let whitespace_re = regex!(r"^(\s+)");

        if view.is_empty() {
            if self.terminated {
                return Err(TokenizerError::new(
                    "Cannot read after EOF".into(),
                    start_index,
                    start_index,
                ));
            } else {
                return Ok(TokenNode::new(Token::Eof, start_index, start_index + 1));
            }
        } else if let Some(cap) = whitespace_re.captures(view) {
            return self.read(start_index + cap[0].len());
        }

        for rule in &self.rules {
            match rule {
                TokenizerRule::Char(ch, token) => {
                    if view.starts_with(*ch) {
                        return Ok(TokenNode::new(token.clone(), start_index, start_index + 1));
                    }
                }
                TokenizerRule::String(str, token) => {
                    if view.starts_with(str) {
                        return Ok(TokenNode::new(
                            token.clone(),
                            start_index,
                            start_index + str.len(),
                        ));
                    }
                }
                TokenizerRule::Regex(regex, factory) => {
                    if let Some(cap) = regex.captures(view) {
                        if !self.matches_keyword.is_match(&cap[0]) {
                            return Ok(TokenNode::new(
                                factory(&cap),
                                start_index,
                                start_index + cap[0].len(),
                            ));
                        }
                    }
                }
            }
        }

        Err(TokenizerError::new(
            "Unexpected token!".into(),
            start_index,
            start_index + view.len(),
        ))
    }

    fn advance(&mut self) -> TokenResult {
        let result = self.read(self.cursor);

        self.cursor = match result {
            Ok(ref source) => source.span.end,
            Err(ref source) => source.span.end,
        };

        if let Ok(TokenNode { ref token, .. }) = result {
            if *token == Token::Eof && self.cursor == self.data.len() {
                self.terminated = true;
            }
        }

        result
    }
}

impl Tokenizer for SimpleTokenizer<'_> {
    fn next(&mut self) -> TokenResult {
        if let Some(next_token) = self.next.take() {
            return next_token;
        }

        self.advance()
    }

    fn peek(&mut self) -> TokenResult {
        if let Some(ref next) = self.next {
            return next.clone();
        }

        let result = self.advance();

        self.next = Some(result.clone());

        result
    }

    fn collect_tokens(&mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut result = Vec::new();

        let mut eof = false;
        while !eof {
            let next = self.next()?;
            eof = next.token == Token::Eof;
            result.push(next.token);
        }

        Ok(result)
    }

    fn get_empty_span(&mut self) -> Result<Span, TokenizerError> {
        let start = self.peek()?.span.start;

        Ok(Span { start, end: start })
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{SimpleTokenizer, Token, Tokenizer};

    #[test]
    fn empty() {
        let mut tokenizer = SimpleTokenizer::new("");
        assert_eq!(tokenizer.collect_tokens().unwrap(), [Token::Eof]);
    }

    #[test]
    fn addition() {
        let mut tokenizer = SimpleTokenizer::new("1 + 2");
        assert_eq!(
            tokenizer.collect_tokens().unwrap(),
            [
                Token::Number(1),
                Token::Operator(crate::lexer::Operator::Plus),
                Token::Number(2),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn peek() {
        let mut tokenizer = SimpleTokenizer::new("1 asd");

        assert_eq!(tokenizer.peek().unwrap(), Token::Number(1));
        tokenizer.next().unwrap();
        assert_eq!(tokenizer.peek().unwrap(), Token::Identifier("asd".into()));
        assert_eq!(tokenizer.peek().unwrap(), Token::Identifier("asd".into()));
    }
}
