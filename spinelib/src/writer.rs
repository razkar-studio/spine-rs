pub fn to_string(value: &spine_rs::Value) -> String {
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
                buf.push_str(&format!("{pipes}{k} = null\n"));
            }
        }
        spine_rs::Value::Bool(b) => {
            if let Some(k) = key {
                buf.push_str(&format!("{pipes}{k} = {b}\n"));
            }
        }
        spine_rs::Value::Number(n) => {
            if let Some(k) = key {
                let s = if n.fract() == 0.0 {
                    format!("{n:.1}")
                } else {
                    format!("{n}")
                };
                buf.push_str(&format!("{pipes}{k} = {s}\n"));
            }
        }
        spine_rs::Value::String(s) => {
            if let Some(k) = key {
                buf.push_str(&format!("{pipes}{k} = {s}\n"));
            }
        }
        spine_rs::Value::Tagged(tag, content) => {
            if let Some(k) = key {
                buf.push_str(&format!("{pipes}{k} = {tag}\"{content}\"\n"));
            }
        }
        spine_rs::Value::Object(fields) => {
            if let Some(k) = key {
                buf.push_str(&format!("{pipes}{k}\n"));
            }
            for (field_key, field_value) in fields {
                write_value(field_value, buf, depth + 1, Some(field_key));
            }
        }
        spine_rs::Value::Array(items) => {
            if let Some(k) = key {
                buf.push_str(&format!("{pipes}{k}\n"));
            }
            for item in items {
                match item {
                    spine_rs::Value::Object(fields) => {
                        buf.push_str(&format!("{pipes}| -\n"));
                        for (field_key, field_value) in fields {
                            write_value(field_value, buf, depth + 2, Some(field_key));
                        }
                    }
                    _ => {
                        let scalar = scalar_to_string(item);
                        buf.push_str(&format!("{pipes}| - {scalar}\n"));
                    }
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
