use evaluator::Evaluator;
use lexer::SimpleTokenizer;
use parser::Parser;
use std::env;
use std::fs;
use utils::format_error;

mod evaluator;
mod lexer;
mod parser;
mod runtime;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_path: String = "example.txt".into();
    let path = args.get(1).unwrap_or(&default_path);

    let content = match fs::read_to_string(path) {
        Ok(str) => str,
        Err(err) => return println!("Error reading file: {}", err),
    };

    let tokenizer = SimpleTokenizer::new(&content);
    let mut parser = Parser::new(tokenizer);

    let program = match parser.parse() {
        Ok(program) => program,
        Err(err) => {
            println!("{}", format_error(&err, &content));
            return;
        }
    };

    println!("Starting...");

    let evaluator = Evaluator {};
    match evaluator.evaluate(program) {
        Ok(()) => println!("Success"),
        Err(err) => println!("{}", format_error(&err, &content)),
    }
}
