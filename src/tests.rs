use crate::{Lexer, Parser, Value};

#[test]
fn test_basic_lexer() {
    let mut lexer = Lexer::new("server\n| host = localhost\n| port = 8080\n");
    let tokens = lexer.tokenize();
    println!("{tokens:?}");
}

#[test]
fn test_basic_object() {
    let src = "server\n| host = localhost\n| port = 8080\n";
    let tokens = Lexer::new(src).tokenize();
    // println!("{tokens:?}");
    let value = Parser::new(tokens).parse();
    println!("{value:?}");

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
    // println!("{tokens:?}");
    let value = Parser::new(tokens).parse();
    println!("{value:?}");

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

#[test]
fn test_array_block() {
    let src = "features\n| - auth\n| - sync\n| - metrics\n";
    let tokens = Lexer::new(src).tokenize();
    let value = Parser::new(tokens).parse();
    println!("{value:?}");

    if let Value::Object(fields) = value {
        if let Value::Array(arr) = &fields[0].1 {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("expected array");
        }
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_array_objects() {
    let src = "array\n| -\n| | message = \"Works\"\n| -\n| | message = \"yeah\"\n";
    let tokens = Lexer::new(src).tokenize();
    let value = Parser::new(tokens).parse();
    println!("{value:?}");

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

#[test]
fn test_multiline_string() {
    let src = "server\n| query = \"\"\"\n| | SELECT *\n| | FROM users\n| | \"\"\"\n";
    let tokens = Lexer::new(src).tokenize();
    let value = Parser::new(tokens).parse();
    println!("{value:?}");

    if let Value::Object(fields) = value {
        if let Value::Object(inner) = &fields[0].1 {
            if let Value::String(s) = &inner[0].1 {
                assert!(s.contains("SELECT *"), "got: {}", s);
                assert!(s.contains("FROM users"), "got: {}", s);
            } else {
                panic!("expected string");
            }
        } else {
            panic!("expected inner object");
        }
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_tagged_literals() {
    let src = "created = date\"2026-05-26\"\nexpires = std.date\"2027-01-01\"\n";
    let tokens = Lexer::new(src).tokenize();
    let value = Parser::new(tokens).parse();
    println!("{value:?}");

    if let Value::Object(fields) = value {
        assert!(matches!(&fields[0].1, Value::Tagged(t, _) if t == "date"));
        assert!(matches!(&fields[1].1, Value::Tagged(t, _) if t == "std.date"));
    } else {
        panic!("expected object");
    }
}
