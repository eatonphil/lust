mod lex;
mod parse;
mod eval;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let contents = fs::read_to_string(&args[1])
        .expect("Could not read file");

    let raw: Vec<char> = contents.to_string().chars().collect();

    println!("Before lexing");
    let tokens = match lex::lex(&raw) {
	Ok(tokens) => tokens,
	Err(msg) => panic!("{}", msg),
    };
    println!("{:#?}", tokens);

    println!("After lexing, before parsing");
    let ast = match parse::parse(&raw, tokens) {
	Ok(ast) => ast,
	Err(msg) => panic!("{}", msg),
    };

    println!("After parsing, before compiling");
    let pgrm = eval::compile(&raw, ast);

    println!("After compiling, before eval");
    eval::eval(pgrm);
}
