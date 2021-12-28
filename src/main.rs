mod lex;
mod parse;
mod eval;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let contents = fs::read_to_string(args[0])
        .expect("Could not read file");

    let tokens = match lex(contents) {
	Ok(tokens) => tokens,
	Err(msg) => panic!("{}", msg),
    };

    let ast = match parse(contents, tokens) {
	Ok(ast) => ast,
	Err(msg) => panic!("{}", msg),
    }

    let pgrm = compile(contents, ast);

    eval(contents, ast);
}
