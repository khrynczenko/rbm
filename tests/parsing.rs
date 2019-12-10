use std::fs;
use rbm::scanner;
use rbm::parser;

#[test]
fn test_parsing_good() {
    let directories = fs::read_dir("./tests/parser_programs/good").unwrap();
    for directory in directories {
        let canonicalized = directory.unwrap().path();
        println!("XDDDDDDDDDDDDDDDDDD - {:?}", canonicalized);
        let content: String = fs::read_to_string(canonicalized).unwrap();
        let tokens = scanner::tokenize(content.as_str()).unwrap();
        println!("tokenized");
        let ast = parser::parse(&tokens);
        let b = ast.is_ok();
        if ast.is_err() {
            println!("{:?}", ast.unwrap_err());
        }
        assert!(b)
    }
}

#[test]
fn test_parsing_bad() {
    let directories = fs::read_dir("./tests/parser_programs/bad").unwrap();
    for directory in directories {
        let canonicalized = directory.unwrap().path();
        println!("XDDDDDDDDDDDDDDDDDD - {:?}", canonicalized);
        let content: String = fs::read_to_string(canonicalized).unwrap();
        let tokens = scanner::tokenize(content.as_str()).unwrap();
        let ast = parser::parse(&tokens);
        let b = ast.is_err();
        if ast.is_ok() {
            println!("{:?}", ast.unwrap());
        }
        assert!(b);
    }
}
