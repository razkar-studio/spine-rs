use super::*;
use crate::document::DocError;
use std::{path::PathBuf, str::FromStr};

#[test]
fn test_ffi_traverse() {
    let src = "server\n| host = localhost\n| port = 8080\n";
    let doc = Document::from_str_or_panic(src);
    let root = doc.root().expect("no root");
    let server = root.get("server").expect("no server");
    let host = server.get("host").expect("no host");
    let port = server.get("port").expect("no port");
    assert_eq!(host.as_str().unwrap(), "localhost");
    assert!((port.as_f64().unwrap() - 8080.0).abs() < f64::EPSILON);
}

#[test]
fn test_ffi_errors_survive_boundary() {
    let src = "host = localhost\nhost = example.com\n";
    let result = Document::from_str(src);
    assert!(result.is_err());
    let DocError::Parse(errors) = result.unwrap_err() else {
        panic!("expected parse error");
    };
    assert!(!errors.is_empty());
    assert!(errors[0].contains("duplicate-key"));
    println!("{}", errors[0]);
}

#[test]
fn test_ffi_multiple_errors_survive_boundary() {
    let src = "age = 18\nemail = alice@tuta.io\n\nage = 12\nemail = alice@example.com";
    let result = Document::from_str(src);
    assert!(result.is_err());
    let DocError::Parse(errors) = result.unwrap_err() else {
        panic!("expected parse error");
    };
    assert!(!errors.is_empty());
    println!("{}", errors[0]);
}

#[test]
fn test_ffi_filename_in_error() {
    let path = PathBuf::from_str("invalid.spn").unwrap();
    let result = Document::from_path(path);
    assert!(result.is_err());
    let DocError::Parse(errors) = result.unwrap_err() else {
        panic!("expected parse error");
    };
    assert!(!errors.is_empty());
    assert!(errors[0].contains("invalid.spn"), "got: {}", errors[0]);
    println!("{}", errors[0]);
}

#[test]
fn test_ffi_from_file() {
    let path = PathBuf::from_str("../example.spn").unwrap();
    let value = Document::from_path(path);
    assert!(value.is_ok());
    println!("{:?}", value.unwrap());
}

#[test]
fn test_format_details() {
    let details = format_details();
    assert!(!details.version.is_empty(), "version should not be empty");
    assert!(!details.spec.is_empty(), "spec should not be empty");
    assert_eq!(details.backend, "native");
    println!("{:?}", details);
}

#[test]
fn test_parse_to_json_success() {
    let json = parse_to_json("server\n| host = localhost\n| port = 8080\n");
    assert!(json.contains("\"ok\":true"), "expected success, got: {json}");
    assert!(json.contains("localhost"), "expected localhost, got: {json}");
    assert!(json.contains("8080"), "expected 8080, got: {json}");
    assert!(json.contains("server"), "expected server, got: {json}");
    assert!(json.contains("host"), "expected host, got: {json}");
    assert!(json.contains("port"), "expected port, got: {json}");
    assert!(json.contains("\"backend\":\"native\""));
    assert!(json.contains("version"));
    assert!(json.contains("spec"));
    println!("{json}");
}

#[test]
fn test_parse_to_json_errors() {
    let json = parse_to_json("host = localhost\nhost = example.com\n");
    assert!(json.contains("\"ok\":false"), "expected failure, got: {json}");
    assert!(json.contains("duplicate-key"), "expected duplicate-key, got: {json}");
    assert!(json.contains("\"value\":null"), "expected null value, got: {json}");
    println!("{json}");
}

#[test]
fn test_parse_to_json_empty() {
    let json = parse_to_json("");
    assert!(json.contains("ok"), "expected ok field, got: {json}");
    println!("{json}");
}

#[test]
fn test_ffi_dotted_append() {
    let src =
        "server\n| host = localhost\n~server.users\n| name = alice\n~server.users\n| name = bob\n";
    let doc = Document::from_str_or_panic(src);
    let root = doc.root().expect("no root");
    let server = root.get("server").expect("no server");
    let users = server.get("users").expect("no users array");
    assert_eq!(users.len(), 2);
    let alice = users.get_index(0).expect("no alice");
    let bob = users.get_index(1).expect("no bob");
    assert_eq!(alice.get("name").unwrap().as_str().unwrap(), "alice");
    assert_eq!(bob.get("name").unwrap().as_str().unwrap(), "bob");
}
