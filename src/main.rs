use evaluator::Evaluator;
use lexer::SimpleTokenizer;
use parser::Parser;
use std::env;
use std::fs;
use utils::format_error;
use utils::MainError;

mod evaluator;
mod lexer;
mod parser;
mod runtime;
mod utils;

fn main() -> Result<(), MainError> {
    const DEFAULT_PATH: &str = "example.txt";
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).map(String::as_str).unwrap_or(DEFAULT_PATH);

    let content = fs::read_to_string(path).map_err(|err| format!("Can not read file: {err}"))?;

    let tokenizer = SimpleTokenizer::new(&content);
    let mut parser = Parser::new(tokenizer);

    println!("Parsing...");

    let program = parser
        .parse()
        .map_err(|ref err| format_error(err, &content))?;

    println!("Starting...");

    let evaluator = Evaluator {};
    evaluator
        .evaluate(program)
        .map_err(|ref err| format_error(err, &content))?;

    println!("Success!");

    Ok(())
}
