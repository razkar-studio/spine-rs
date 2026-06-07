use std::fmt::Write;

pub(crate) fn to_string_inner(value: &spine_rs::Value) -> String {
    let mut buf = String::new();
    match value {
        spine_rs::Value::Object(fields) => {
            for (key, val) in fields {
                write_value(val, &mut buf, 0, Some(key));
            }
        }
        _ => write_value(value, &mut buf, 0, None),
    }
    buf
}

fn write_value(value: &spine_rs::Value, buf: &mut String, depth: usize, key: Option<&str>) {
    let pipes = "| ".repeat(depth);
    match value {
        spine_rs::Value::Null => {
            if let Some(k) = key {
                let _ = writeln!(buf, "{pipes}{k} = null");
            }
        }
        spine_rs::Value::Bool(b) => {
            if let Some(k) = key {
                let _ = writeln!(buf, "{pipes}{k} = {b}");
            }
        }
        spine_rs::Value::Number(n) => {
            if let Some(k) = key {
                let s = if n.fract() == 0.0 {
                    format!("{n:.1}")
                } else {
                    format!("{n}")
                };
                let _ = writeln!(buf, "{pipes}{k} = {s}");
            }
        }
        spine_rs::Value::String(s) => {
            if let Some(k) = key {
                let _ = writeln!(buf, "{pipes}{k} = {s}");
            }
        }
        spine_rs::Value::Tagged(tag, content) => {
            if let Some(k) = key {
                let _ = writeln!(buf, "{pipes}{k} = {tag}\"{content}\"");
            }
        }
        spine_rs::Value::Object(fields) => {
            if let Some(k) = key {
                let _ = writeln!(buf, "{pipes}{k}");
            }
            for (field_key, field_value) in fields {
                write_value(field_value, buf, depth + 1, Some(field_key));
            }
        }
        spine_rs::Value::Array(items) => {
            if let Some(k) = key {
                let _ = writeln!(buf, "{pipes}{k}");
            }
            for item in items {
                if let spine_rs::Value::Object(fields) = item {
                    let _ = writeln!(buf, "{pipes}| -");
                    for (field_key, field_value) in fields {
                        write_value(field_value, buf, depth + 2, Some(field_key));
                    }
                } else {
                    let scalar = scalar_to_string(item);
                    let _ = writeln!(buf, "{pipes}| - {scalar}");
                }
            }
        }
    }
}

fn scalar_to_string(value: &spine_rs::Value) -> String {
    match value {
        spine_rs::Value::Null => "null".to_string(),
        spine_rs::Value::Bool(b) => b.to_string(),
        spine_rs::Value::Number(n) => {
            if n.fract() == 0.0 {
                format!("{n:.1}")
            } else {
                format!("{n}")
            }
        }
        spine_rs::Value::String(s) => s.clone(),
        spine_rs::Value::Tagged(tag, content) => format!("{tag}\"{content}\""),
        _ => String::new(),
    }
}
