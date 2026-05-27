use crate::{Lexer, Parser, Value};

#[test]
fn test_basic_lexer() {
    let mut lexer = Lexer::new("server\n| host = localhost\n| port = 8080\n");
    let tokens = lexer.tokenize();
    eprintln!("{tokens:#?}");
}

#[test]
fn test_basic_object() {
    let src = "server\n| host = localhost\n| port = 8080\n";
    let tokens = Lexer::new(src).tokenize();
    // eprintln!("{tokens:?}");
    let value = Parser::new(tokens).parse();
    eprintln!("{value:#?}");

    if let Value::Object(fields) = value {
        assert_eq!(fields[0].0, "server");
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_append() {
    let src = "~packages\n| name = react\n~packages\n| name = vue\n";
    let tokens = Lexer::new(src).tokenize();
    // eprintln!("{tokens:?}");
    let value = Parser::new(tokens).parse();
    eprintln!("{value:#?}");

    if let Value::Object(fields) = value {
        if let Value::Array(arr) = &fields[0].1 {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("expected array");
        }
    } else {
        panic!("expected object");
    }
}
