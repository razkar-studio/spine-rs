use crate::{Lexer, Parser, Value};
use std::{fs, path::PathBuf, str::FromStr};

#[test]
fn test_basic_lexer() {
    let src = "server\n| host = localhost\n| port = 8080\n";
    let tokens = Lexer::new(src).tokenize();
    println!("{tokens:?}");
}

#[test]
fn test_basic_object() {
    let src = "server\n| host = localhost\n| port = 8080\n";
    let tokens = Lexer::new(src).tokenize();
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
    let src = "array\n| -\n| | message = Works\n| -\n| | message = yeah\n";
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
    assert!(errors[0].contains("duplicate-key"));
    println!("{}", errors[0]);
}

#[test]
fn test_type_conflict_error() {
    let src = "server = localhost\nserver\n| port = 8080\n";
    let tokens = Lexer::new(src).tokenize();
    let result = Parser::new(tokens, src).parse();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("type-conflict"));
    println!("{}", errors[0]);
}

#[test]
fn test_append_type_conflict_error() {
    let src = "mode = production\n~mode\n| env = staging\n";
    let tokens = Lexer::new(src).tokenize();
    let result = Parser::new(tokens, src).parse();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("type-conflict"));
    println!("{}", errors[0]);
}

#[test]
fn test_error_accumulation() {
    let src = "server = localhost\nserver\n| port = 8080\nhost = localhost\nhost = example.com\n";
    let tokens = Lexer::new(src).tokenize();
    let result = Parser::new(tokens, src).parse();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 2);
    println!("{}", errors.join(""));
}

#[test]
fn test_unknown_character() {
    let src = r#"{"key": "value"}"#;
    let tokens = Lexer::new(src).tokenize();
    let result = Parser::new(tokens, src).parse();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("unexpected-character"));
    println!("{}", errors[0]);
}

#[test]
fn test_unknown_with_spine_errors() {
    let src = "host = localhost\nhost = example.com\n{wat}";
    let tokens = Lexer::new(src).tokenize();
    let result = Parser::new(tokens, src).parse();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 2);
    println!("{}", errors.join(""));
}

#[test]
fn test_bare_strings() {
    let src = "database\n| host = db.primary.local\n| endpoint = https://telemetry.local\n| password = scram-sha-256\n| window = 60s\n| timestamp = 2026-05-26T00:00:00Z\n";
    let tokens = Lexer::new(src).tokenize();
    let value = Parser::new(tokens, src).parse().expect("parse failed");
    println!("{value:?}");
    if let Value::Object(fields) = value {
        if let Value::Object(inner) = &fields[0].1 {
            for (k, v) in inner {
                println!("{k} = {v:?}");
            }
            assert_eq!(inner[0].1, Value::String("db.primary.local".to_string()));
            assert_eq!(
                inner[1].1,
                Value::String("https://telemetry.local".to_string())
            );
            assert_eq!(inner[2].1, Value::String("scram-sha-256".to_string()));
            assert_eq!(inner[3].1, Value::String("60s".to_string()));
            assert_eq!(
                inner[4].1,
                Value::String("2026-05-26T00:00:00Z".to_string())
            );
        } else {
            panic!("expected inner object");
        }
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_bare_strings_in_array() {
    let src = "regions\n| - eu-central-1\n| - eu-west-1\n| - us-east-1\n";
    let tokens = Lexer::new(src).tokenize();
    let value = Parser::new(tokens, src).parse().expect("parse failed");
    println!("{value:?}");
    if let Value::Object(fields) = value {
        if let Value::Array(arr) = &fields[0].1 {
            for (i, v) in arr.iter().enumerate() {
                println!("[{i}] = {v:?}");
            }
            assert_eq!(arr[0], Value::String("eu-central-1".to_string()));
            assert_eq!(arr[1], Value::String("eu-west-1".to_string()));
            assert_eq!(arr[2], Value::String("us-east-1".to_string()));
        } else {
            panic!("expected array");
        }
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_from_file() {
    let path = PathBuf::from_str("../example.spn").unwrap();
    let src = fs::read_to_string(path).unwrap();
    let tokens = Lexer::new(&src).tokenize();
    let result = Parser::new(tokens, &src).parse();
    if let Err(errors) = result {
        println!("{}", errors.join(""));
        panic!();
    }
}
