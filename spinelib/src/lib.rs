use std::fmt::Write;

mod de;
mod document;
mod ser;
mod value;
mod writer;

pub use de::{DeError, from_document};
pub use document::Document;
pub use ser::{SerError, to_document};
pub use value::{Value, ValueType};

/// Build-time metadata about the parser.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatDetails {
    /// The parser version.
    pub version: String,
    /// The spec version this parser targets.
    pub spec: String,
    /// Whether the backend is native or WASM.
    pub backend: String,
}

/// Returns metadata about the parser.
pub fn format_details() -> FormatDetails {
    FormatDetails {
        version: env!("CARGO_PKG_VERSION").to_string(),
        spec: "1.0-rc.2".to_string(),
        backend: "native".to_string(),
    }
}

/// Parse Spine source and return the AST as a JSON string.
///
/// The JSON output includes format metadata, success status, and either
/// the parsed AST or a list of errors.
pub fn parse_to_json(input: &str) -> String {
    let tokens = spine_rs::Lexer::new(input).tokenize();
    let mut parser = spine_rs::Parser::new(tokens, input);
    let result = parser.parse();

    let mut json = String::with_capacity(4096);
    json.push('{');
    write_json_str("version", &mut json);
    json.push_str(":\"0.1.0\",");
    write_json_str("spec", &mut json);
    json.push_str(":\"1.0-rc.2\",");
    write_json_str("backend", &mut json);
    json.push_str(":\"native\"");

    match result {
        Ok(value) => {
            json.push_str(",\"ok\":true,");
            write_json_str("value", &mut json);
            json.push(':');
            write_json_value(&value, &mut json);
            json.push_str(",\"errors\":[]");
        }
        Err(errors) => {
            json.push_str(",\"ok\":false,\"value\":null,");
            write_json_str("errors", &mut json);
            json.push_str(":[");
            for (i, err) in errors.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                write_json_string(err, &mut json);
            }
            json.push(']');
        }
    }

    json.push('}');
    json
}

fn write_json_str(s: &str, buf: &mut String) {
    buf.push('"');
    buf.push_str(s);
    buf.push('"');
}

fn write_json_value(val: &spine_rs::Value, buf: &mut String) {
    match val {
        spine_rs::Value::Null => buf.push_str("null"),
        spine_rs::Value::Bool(b) => buf.push_str(if *b { "true" } else { "false" }),
        spine_rs::Value::Number(n) => write_json_number(*n, buf),
        spine_rs::Value::String(s) => write_json_string(s, buf),
        spine_rs::Value::Tagged(tag, content) => {
            buf.push_str("{\"tag\":");
            write_json_string(tag, buf);
            buf.push_str(",\"content\":");
            write_json_string(content, buf);
            buf.push('}');
        }
        spine_rs::Value::Array(arr) => {
            buf.push('[');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    buf.push(',');
                }
                write_json_value(v, buf);
            }
            buf.push(']');
        }
        spine_rs::Value::Object(fields) => {
            buf.push('{');
            for (i, (k, v)) in fields.iter().enumerate() {
                if i > 0 {
                    buf.push(',');
                }
                write_json_string(k, buf);
                buf.push(':');
                write_json_value(v, buf);
            }
            buf.push('}');
        }
    }
}

fn write_json_number(n: f64, buf: &mut String) {
    if n.is_finite() {
        let s = format!("{n}");
        buf.push_str(&s);
    } else {
        buf.push('0');
    }
}

fn write_json_string(s: &str, buf: &mut String) {
    buf.push('"');
    for c in s.chars() {
        match c {
            '"' => buf.push_str("\\\""),
            '\\' => buf.push_str("\\\\"),
            '\n' => buf.push_str("\\n"),
            '\t' => buf.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(buf, "\\u{:04x}", c as u32);
            }
            c => buf.push(c),
        }
    }
    buf.push('"');
}

#[cfg(test)]
mod tests;
