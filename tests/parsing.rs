use rbm::parser;
use rbm::scanner;
use std::fs;

#[test]
fn test_parsing_good() {
    let directories = fs::read_dir("./tests/parser_programs/good").unwrap();
    for directory in directories {
        let canonicalized = directory.unwrap().path();
        println!("FILENAME - {:?}", canonicalized);
        let content: String = fs::read_to_string(canonicalized).unwrap();
        println!("Printing content");
        let tokens = scanner::tokenize(content.as_str()).unwrap();
        println!("{}", content.as_str());
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
