use std::path::PathBuf;
use std::fs;
use clap::{Arg, App, SubCommand};
extern crate rbm;
use rbm::scanner;
use rbm::parser;

fn main() {
    let matches = App::new("rbm")
        .about("WIP compiler for b-minor language.")
        .subcommand(SubCommand::with_name("lex")
                    .about("Tokenizes specified source file")
                    .arg(Arg::with_name("file")
                         .required(true)
                         )
                    )
        .subcommand(SubCommand::with_name("parse")
                    .about("Parses specified source file")
                    .arg(Arg::with_name("file")
                         .required(true)
                         )
                    ).get_matches();
    if matches.is_present("lex") {
        let smatches = matches.subcommand_matches("lex").unwrap();
        let source_file_str = smatches.value_of("file").unwrap();
        let source_file_path = PathBuf::from(source_file_str);
        if !source_file_path.exists() {
            println!("Source file {} does not exist.", source_file_str);
            return;
        }
        let content = fs::read_to_string(source_file_path).unwrap();
        let tokens = scanner::tokenize(&content);
        if tokens.is_ok() {
            scanner::print_pretty(&tokens.unwrap());
        } else {
            print!("{:?}", tokens.unwrap_err());
        }
    } else if matches.is_present("parse") {
        let smatches = matches.subcommand_matches("parse").unwrap();
        let source_file_str = smatches.value_of("file").unwrap();
        let source_file_path = PathBuf::from(source_file_str);
        if !source_file_path.exists() {
            println!("Source file {} does not exist.", source_file_str);
            return;
        }
        let content = fs::read_to_string(source_file_path).unwrap();
        let tokens_result = scanner::tokenize(&content);
        match tokens_result {
            Ok(tokens) => {
                let ast = parser::parse(&tokens);
                if ast.is_err() {
                    println!("{:?}", ast.unwrap_err());
                } else {
                    dbg!(ast.unwrap());
                }
            },
            Err(err) => {
                println!("{:?}", err);
            }
        }
    } else {
        print!("{}", matches.usage());
    }
}
