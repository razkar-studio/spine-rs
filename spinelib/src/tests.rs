#![allow(clippy::approx_constant, clippy::float_cmp)]

use super::*;
use crate::document::DocError;
use std::{path::PathBuf, str::FromStr};

// ── Basic parse & traverse ──

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

// ── Value types ──

#[test]
fn test_value_null() {
    let doc = Document::from_str_or_panic("n = null\n");
    let n = doc.root().unwrap().get("n").unwrap();
    assert_eq!(n.value_type(), ValueType::Null);
    assert_eq!(n.as_f64(), None);
}

#[test]
fn test_value_bool() {
    let doc = Document::from_str_or_panic("a = true\nb = false\n");
    let root = doc.root().unwrap();
    assert_eq!(root.get("a").unwrap().value_type(), ValueType::Bool);
    assert_eq!(root.get("a").unwrap().as_str(), None);
}

#[test]
fn test_value_number() {
    let doc = Document::from_str_or_panic("p = 8080\nn = -42\nd = 3.14\n");
    let root = doc.root().unwrap();
    let p = root.get("p").unwrap();
    let n = root.get("n").unwrap();
    let d = root.get("d").unwrap();
    assert!((p.as_f64().unwrap() - 8080.0).abs() < f64::EPSILON);
    assert!((n.as_f64().unwrap() - (-42.0)).abs() < f64::EPSILON);
    assert!((d.as_f64().unwrap() - 3.14).abs() < f64::EPSILON);
    assert_eq!(p.value_type(), ValueType::Number);
}

#[test]
fn test_value_string() {
    let doc = Document::from_str_or_panic("a = hello\nb = \"world\"\nc = 300s\n");
    let root = doc.root().unwrap();
    assert_eq!(root.get("a").unwrap().as_str().unwrap(), "hello");
    assert_eq!(root.get("b").unwrap().as_str().unwrap(), "world");
    assert_eq!(root.get("c").unwrap().as_str().unwrap(), "300s");
    assert_eq!(root.get("a").unwrap().value_type(), ValueType::String);
}

#[test]
fn test_value_tagged() {
    let doc = Document::from_str_or_panic("c = date\"2026-05-26\"\nn = std.date\"2027-01-01\"\n");
    let root = doc.root().unwrap();
    let (tag1, val1) = root.get("c").unwrap().tag().unwrap();
    assert_eq!(tag1, "date");
    assert_eq!(val1, "2026-05-26");
    let (tag2, val2) = root.get("n").unwrap().tag().unwrap();
    assert_eq!(tag2, "std.date");
    assert_eq!(val2, "2027-01-01");
}

#[test]
fn test_value_array() {
    let doc = Document::from_str_or_panic("arr\n| - a\n| - b\n| - c\n");
    let root = doc.root().unwrap();
    let arr = root.get("arr").unwrap();
    assert_eq!(arr.value_type(), ValueType::Array);
    assert_eq!(arr.len(), 3);
    assert!(!arr.is_empty());
    assert_eq!(arr.get_index(0).unwrap().as_str().unwrap(), "a");
    assert_eq!(arr.get_index(1).unwrap().as_str().unwrap(), "b");
    assert_eq!(arr.get_index(2).unwrap().as_str().unwrap(), "c");
    assert!(arr.get_index(99).is_none());
}

#[test]
fn test_value_array_numbers() {
    let doc = Document::from_str_or_panic("nums\n| - 10\n| - 20\n| - 30\n");
    let arr = doc.root().unwrap().get("nums").unwrap();
    assert_eq!(arr.len(), 3);
    assert!((arr.get_index(0).unwrap().as_f64().unwrap() - 10.0).abs() < f64::EPSILON);
    assert!((arr.get_index(2).unwrap().as_f64().unwrap() - 30.0).abs() < f64::EPSILON);
}

#[test]
fn test_value_array_of_objects() {
    let src = "items\n| -\n| | n = first\n| -\n| | n = second\n| -\n| | n = third\n";
    let doc = Document::from_str_or_panic(src);
    let arr = doc.root().unwrap().get("items").unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(
        arr.get_index(0)
            .unwrap()
            .get("n")
            .unwrap()
            .as_str()
            .unwrap(),
        "first"
    );
    assert_eq!(
        arr.get_index(1)
            .unwrap()
            .get("n")
            .unwrap()
            .as_str()
            .unwrap(),
        "second"
    );
    assert_eq!(
        arr.get_index(2)
            .unwrap()
            .get("n")
            .unwrap()
            .as_str()
            .unwrap(),
        "third"
    );
}

#[test]
fn test_value_object() {
    let src = "server\n| host = localhost\n| port = 8080\n";
    let doc = Document::from_str_or_panic(src);
    let obj = doc.root().unwrap().get("server").unwrap();
    assert_eq!(obj.value_type(), ValueType::Object);
    assert_eq!(obj.len(), 2);
    assert!(!obj.is_empty());
    assert_eq!(obj.key_at(0).unwrap(), "host");
    assert_eq!(obj.key_at(1).unwrap(), "port");
}

// ── Edge cases ──

#[test]
fn test_deeply_nested() {
    let src = "a\n| b\n| | c\n| | | d = deep\n";
    let doc = Document::from_str_or_panic(src);
    let val = doc
        .root()
        .unwrap()
        .get("a")
        .unwrap()
        .get("b")
        .unwrap()
        .get("c")
        .unwrap()
        .get("d")
        .unwrap();
    assert_eq!(val.as_str().unwrap(), "deep");
}

#[test]
fn test_empty_object() {
    let doc = Document::from_str_or_panic("e = null\n");
    let obj = doc.root().unwrap();
    assert_eq!(obj.len(), 1);
}

#[test]
fn test_empty_array_value() {
    let doc = Document::from_str_or_panic("arr\n| -\n| -\n");
    let arr = doc.root().unwrap().get("arr").unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr.get_index(0).unwrap().value_type(), ValueType::Null);
    assert_eq!(arr.get_index(1).unwrap().value_type(), ValueType::Null);
}

#[test]
fn test_missing_key_returns_none() {
    let doc = Document::from_str_or_panic("a = 1\n");
    assert!(doc.root().unwrap().get("nonexistent").is_none());
}

#[test]
fn test_get_on_non_object_returns_none() {
    let doc = Document::from_str_or_panic("s = hello\n");
    let s = doc.root().unwrap().get("s").unwrap();
    assert!(s.get("anything").is_none());
}

#[test]
fn test_get_index_on_non_array_returns_none() {
    let doc = Document::from_str_or_panic("s = hello\n");
    let s = doc.root().unwrap().get("s").unwrap();
    assert!(s.get_index(0).is_none());
}

#[test]
fn test_bare_strings() {
    let src = "cfg\n| host = db.primary.local\n| url = https://example.com\n| ttl = 60s\n";
    let doc = Document::from_str_or_panic(src);
    let cfg = doc.root().unwrap().get("cfg").unwrap();
    assert_eq!(
        cfg.get("host").unwrap().as_str().unwrap(),
        "db.primary.local"
    );
    assert_eq!(
        cfg.get("url").unwrap().as_str().unwrap(),
        "https://example.com"
    );
    assert_eq!(cfg.get("ttl").unwrap().as_str().unwrap(), "60s");
}

#[test]
fn test_bare_strings_in_array() {
    let src = "regions\n| - eu-central-1\n| - eu-west-1\n| - us-east-1\n";
    let doc = Document::from_str_or_panic(src);
    let arr = doc.root().unwrap().get("regions").unwrap();
    assert_eq!(arr.get_index(0).unwrap().as_str().unwrap(), "eu-central-1");
    assert_eq!(arr.get_index(1).unwrap().as_str().unwrap(), "eu-west-1");
    assert_eq!(arr.get_index(2).unwrap().as_str().unwrap(), "us-east-1");
}

#[test]
fn test_multiline_string() {
    let src = "q = \"\"\"\n| | hello\n| | world\n| | \"\"\"\n";
    let doc = Document::from_str_or_panic(src);
    let val = doc.root().unwrap().get("q").unwrap();
    assert_eq!(val.as_str().unwrap(), "hello\nworld");
}

#[test]
fn test_comments_ignored() {
    let src = "# this is a comment\na = 1\n# another comment\n";
    let doc = Document::from_str_or_panic(src);
    assert_eq!(doc.root().unwrap().get("a").unwrap().as_f64().unwrap(), 1.0);
}

#[test]
fn test_str_from_empty() {
    let doc = Document::from_str_or_panic("");
    assert_eq!(doc.root().unwrap().len(), 0);
}

#[test]
fn test_key_at_out_of_bounds() {
    let doc = Document::from_str_or_panic("a = 1\n");
    assert!(doc.root().unwrap().key_at(99).is_none());
}

// ── Error cases ──

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
fn test_type_conflict_scalar_vs_object() {
    let src = "s = localhost\ns\n| p = 8080\n";
    let result = Document::from_str(src);
    assert!(result.is_err());
    let DocError::Parse(errors) = result.unwrap_err() else {
        panic!("expected parse error");
    };
    assert!(errors[0].contains("type-conflict"));
}

#[test]
fn test_append_type_conflict() {
    let src = "m = prod\n~m\n| e = staging\n";
    let result = Document::from_str(src);
    assert!(result.is_err());
    let DocError::Parse(errors) = result.unwrap_err() else {
        panic!("expected parse error");
    };
    assert!(errors[0].contains("type-conflict"));
}

#[test]
fn test_unterminated_string_error() {
    let src = "k = \"hello\n";
    let result = Document::from_str(src);
    assert!(result.is_err());
}

#[test]
fn test_file_not_found_error() {
    let path = PathBuf::from_str("nonexistent-file.spn").unwrap();
    let result = Document::from_path(path);
    assert!(result.is_err());
    match result.unwrap_err() {
        DocError::Io(_) => {} // expected
        other @ DocError::Parse(_) => panic!("expected Io error, got {other:?}"),
    }
}

#[test]
fn test_error_accumulation() {
    let src = "s = localhost\ns\n| p = 8080\nh = a\nh = b\n";
    let result = Document::from_str(src);
    assert!(result.is_err());
    let DocError::Parse(errors) = result.unwrap_err() else {
        panic!("expected parse error");
    };
    assert!(errors.len() >= 2);
}

// ── format_details ──

#[test]
fn test_format_details() {
    let details = format_details();
    assert!(!details.version.is_empty(), "version should not be empty");
    assert!(!details.spec.is_empty(), "spec should not be empty");
    assert_eq!(details.backend, "native");
    println!("{details:?}");
}

// ── parse_to_json ──

#[test]
fn test_parse_to_json_success() {
    let json = parse_to_json("server\n| host = localhost\n| port = 8080\n");
    assert!(
        json.contains("\"ok\":true"),
        "expected success, got: {json}"
    );
    assert!(
        json.contains("localhost"),
        "expected localhost, got: {json}"
    );
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
    assert!(
        json.contains("\"ok\":false"),
        "expected failure, got: {json}"
    );
    assert!(
        json.contains("duplicate-key"),
        "expected duplicate-key, got: {json}"
    );
    assert!(
        json.contains("\"value\":null"),
        "expected null value, got: {json}"
    );
    println!("{json}");
}

#[test]
fn test_parse_to_json_empty() {
    let json = parse_to_json("");
    assert!(json.contains("ok"), "expected ok field, got: {json}");
    println!("{json}");
}

#[test]
fn test_parse_to_json_tagged() {
    let json = parse_to_json("c = date\"2026-05-26\"\n");
    assert!(json.contains("\"tag\":\"date\""));
    assert!(json.contains("2026-05-26"));
}

#[test]
fn test_parse_to_json_array() {
    let json = parse_to_json("a\n| - x\n| - y\n| - z\n");
    assert!(json.contains("\"ok\":true"));
}

#[test]
fn test_parse_to_json_nested_object() {
    let json = parse_to_json("a\n| b\n| | c = deep\n");
    assert!(json.contains("deep"));
}

#[test]
fn test_parse_to_json_numbers() {
    let json = parse_to_json("p = 8080\nn = -42\nd = 3.14\n");
    assert!(json.contains("8080"));
    assert!(json.contains("-42"));
    assert!(json.contains("3.14"));
}

#[test]
fn test_parse_to_json_bools() {
    let json = parse_to_json("t = true\nf = false\nn = null\n");
    assert!(json.contains("true"));
    assert!(json.contains("false"));
    assert!(json.contains("null"));
}

// ── Additional edge cases ──

#[test]
fn test_value_type_tagged() {
    let doc = Document::from_str_or_panic("c = date\"2026-01-01\"\n");
    let v = doc.root().unwrap().get("c").unwrap();
    assert_eq!(v.value_type(), ValueType::Tagged);
}

#[test]
fn test_value_type_null() {
    let doc = Document::from_str_or_panic("arr\n| -\n| -\n");
    let arr = doc.root().unwrap().get("arr").unwrap();
    assert_eq!(arr.get_index(0).unwrap().value_type(), ValueType::Null);
}

#[test]
fn test_is_empty_on_scalar() {
    let doc = Document::from_str_or_panic("s = hello\n");
    let s = doc.root().unwrap().get("s").unwrap();
    assert!(s.is_empty());
}

#[test]
fn test_is_empty_on_array() {
    let doc = Document::from_str_or_panic("a\n| - x\n");
    let a = doc.root().unwrap().get("a").unwrap();
    assert!(!a.is_empty());
}

#[test]
fn test_is_empty_on_object() {
    let doc = Document::from_str_or_panic("o\n| k = v\n");
    let o = doc.root().unwrap().get("o").unwrap();
    assert!(!o.is_empty());
}

#[test]
fn test_nested_dotted_access() {
    let src = "a.b.c = deep\n";
    let doc = Document::from_str_or_panic(src);
    let root = doc.root().unwrap();
    let c = root.get("a").unwrap().get("b").unwrap().get("c").unwrap();
    assert_eq!(c.as_str().unwrap(), "deep");
}

#[test]
fn test_append_multiple_via_wrapper() {
    let src = "~items\n| n = first\n~items\n| n = second\n~items\n| n = third\n";
    let doc = Document::from_str_or_panic(src);
    let items = doc.root().unwrap().get("items").unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(
        items
            .get_index(0)
            .unwrap()
            .get("n")
            .unwrap()
            .as_str()
            .unwrap(),
        "first"
    );
    assert_eq!(
        items
            .get_index(2)
            .unwrap()
            .get("n")
            .unwrap()
            .as_str()
            .unwrap(),
        "third"
    );
}

#[test]
fn test_object_merge_duplicate_objects() {
    let src = "obj\n| a = 1\nobj\n| b = 2\n";
    let doc = Document::from_str_or_panic(src);
    let obj = doc.root().unwrap().get("obj").unwrap();
    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("a").unwrap().as_f64().unwrap(), 1.0);
    assert_eq!(obj.get("b").unwrap().as_f64().unwrap(), 2.0);
}

#[test]
fn test_key_at_on_root() {
    let doc = Document::from_str_or_panic("z = 1\na = 2\n");
    assert_eq!(doc.root().unwrap().key_at(0).unwrap(), "z");
    assert_eq!(doc.root().unwrap().key_at(1).unwrap(), "a");
}

#[test]
fn test_dotted_path_wrapper() {
    let src = "a.b.c\n| d = val\n";
    let doc = Document::from_str_or_panic(src);
    let val = doc
        .root()
        .unwrap()
        .get("a")
        .unwrap()
        .get("b")
        .unwrap()
        .get("c")
        .unwrap()
        .get("d")
        .unwrap();
    assert_eq!(val.as_str().unwrap(), "val");
}

#[test]
fn test_empty_quoted_string_value() {
    let doc = Document::from_str_or_panic("e = \"\"\n");
    assert_eq!(doc.root().unwrap().get("e").unwrap().as_str().unwrap(), "");
}

#[test]
fn test_doc_root_on_empty_document() {
    let doc = Document::from_str_or_panic("");
    assert!(doc.root().is_some());
    assert_eq!(doc.root().unwrap().len(), 0);
}

// ── Mixed / integration ──

#[test]
fn test_traverse_example_file() {
    let doc = Document::from_str_or_panic(include_str!("../../example.spn"));
    let root = doc.root().unwrap();
    let app = root.get("app").unwrap();
    assert_eq!(
        app.get("name").unwrap().as_str().unwrap(),
        "Spine Showcase System"
    );
    let system = root.get("system").unwrap();
    let runtime = system.get("runtime").unwrap();
    assert_eq!(runtime.get("env").unwrap().as_str().unwrap(), "production");
    let telemetry = system.get("telemetry").unwrap();
    assert_eq!(
        telemetry.get("endpoint").unwrap().as_str().unwrap(),
        "https://telemetry.local"
    );
    let features = root.get("features").unwrap();
    let flags = features.get("flags").unwrap();
    assert_eq!(flags.len(), 2);
    assert_eq!(
        flags
            .get_index(0)
            .unwrap()
            .get("name")
            .unwrap()
            .as_str()
            .unwrap(),
        "new-ui"
    );
    let meta = root.get("meta").unwrap();
    let events = meta.get("events").unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(
        events
            .get_index(0)
            .unwrap()
            .get("type")
            .unwrap()
            .as_str()
            .unwrap(),
        "created"
    );
}

#[test]
fn test_parse_to_json_full_example() {
    let json = parse_to_json(include_str!("../../example.spn"));
    assert!(json.contains("\"ok\":true"), "expected success: got {json}");
    assert!(json.contains("Spine Showcase System"));
    assert!(json.contains("db.primary.local"));
    assert!(json.contains("std.date"));
    assert!(json.contains("base64"));
}

// --- serde integration --- //

#[test]
fn test_serde_basic() {
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        version: f64,
        enabled: bool,
    }

    let doc = crate::Document::from_str_or_panic(
        r#"
        name = "odyn"
        version = 1.0
        enabled = true
    "#,
    );

    let config: Config = crate::from_document(&doc).unwrap();
    assert_eq!(config.name, "odyn");
    assert_eq!(config.version, 1.0);
    assert!(config.enabled);
}

#[test]
fn test_serde_nested() {
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Lockfile {
        dep: Vec<Dep>,
    }

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Dep {
        name: String,
        source: String,
        commit: String,
    }

    let doc = crate::Document::from_str_or_panic(
        r"
        ~dep
        | name = odin-http
        | source = https://github.com/laytan/odin-http
        | commit = abc123
    ",
    );

    let lockfile: Lockfile = crate::from_document(&doc).unwrap();
    assert_eq!(lockfile.dep.len(), 1);
    assert_eq!(lockfile.dep[0].name, "odin-http");
}

#[test]
fn test_serde_roundtrip() {
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        version: f64,
        enabled: bool,
        tags: Vec<String>,
    }

    let original = Config {
        name: "odyn".to_string(),
        version: 1.0,
        enabled: true,
        tags: vec!["cli".to_string(), "odin".to_string()],
    };

    let doc = crate::to_document(&original).unwrap();
    println!("{}", doc.to_string().unwrap());
    let result: Config = crate::from_document(&doc).unwrap();
    assert_eq!(original, result);
}
