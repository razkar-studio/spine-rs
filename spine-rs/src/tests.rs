use std::{fs, path::PathBuf, str::FromStr};

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
    let value = Parser::new(tokens, src).parse().expect("parse failed");
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
    let value = Parser::new(tokens, src).parse().expect("parse failed");
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
    let value = Parser::new(tokens, src).parse().expect("parse failed");
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
    let value = Parser::new(tokens, src).parse().expect("parse failed");
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
    let value = Parser::new(tokens, src).parse().expect("parse failed");
    println!("{value:?}");

    if let Value::Object(fields) = value {
        if let Value::Object(inner) = &fields[0].1 {
            if let Value::String(s) = &inner[0].1 {
                assert!(s.contains("SELECT *"), "got: {s}");
                assert!(s.contains("FROM users"), "got: {s}");
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
    let value = Parser::new(tokens, src).parse().expect("parse failed");
    println!("{value:?}");

    if let Value::Object(fields) = value {
        assert!(matches!(&fields[0].1, Value::Tagged(t, _) if t == "date"));
        assert!(matches!(&fields[1].1, Value::Tagged(t, _) if t == "std.date"));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_duplicate_key_error() {
    let src = "host = localhost\nhost = example.com\n";
    let tokens = Lexer::new(src).tokenize();
    let result = Parser::new(tokens, src).parse();

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    println!("{}", errors[0]);
}

#[test]
fn test_from_file() {
    let path = PathBuf::from_str("../example.spn").unwrap();
    let src = fs::read_to_string(path).unwrap();
    let tokens = Lexer::new(&src).tokenize();
    let result = Parser::new(tokens, &src).parse();

    if let Err(errors) = result {
        println!("{}", errors.join("\n"));
        panic!()
    }
}

#[test]
fn test_json_input() {
    let src = r#"{"key": "value", "number": 42, "nested": {"a": true}}"#;
    let tokens = Lexer::new(src).tokenize();
    println!("{tokens:?}");
}

#[test]
fn test_unknown_characters() {
    let src = r#"{"key": "value", "number": 42}"#;
    let tokens = Lexer::new(src).tokenize();
    let result = Parser::new(tokens, src).parse();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    println!("{}", errors.join(""));
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("unexpected-character"));
}

#[test]
fn test_unknown_with_spine_errors() {
    let src = "host = localhost\nhost = example.com\n{wat}";
    let tokens = Lexer::new(src).tokenize();
    let result = Parser::new(tokens, src).parse();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    println!("{}", errors.join(""));
    assert_eq!(errors.len(), 2);
}
