use std::fs;
use rbm::scanner;

#[test]
fn test_scanning_good() {
    let directories = fs::read_dir("./tests/scanner_programs/correct").unwrap();
    for directory in directories {
        let canonicalized = directory.unwrap().path();
        let content: String = fs::read_to_string(canonicalized).unwrap();
        let tokens = scanner::tokenize(content.as_str());
        assert!(tokens.is_ok());
    }
}

#[test]
fn test_scanning_bad() {
    let directories = fs::read_dir("./tests/scanner_programs/bad").unwrap();
    for directory in directories {
        let canonicalized = directory.unwrap().path();
        println!("XDDDDDDDDDDDDDDDDDD - {:?}", canonicalized);
        let content: String = fs::read_to_string(canonicalized).unwrap();
        let tokens = scanner::tokenize(content.as_str());
        assert!(tokens.is_err());
    }
}
