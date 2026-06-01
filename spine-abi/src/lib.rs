use spine_rs::{Lexer, Parser, Value};
use std::ffi::{CStr, CString};
use std::fmt::Write;
use std::os::raw::{c_char, c_double, c_int, c_ulong};

/// Opaque types that C sees as pointers
pub struct SpineDoc {
    root: Option<Value>,
    errors: Vec<String>,
}

pub struct SpineValue {
    inner: *const Value,
}

/// # Safety
///
/// `input` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_parse(input: *const c_char) -> *mut SpineDoc {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };
    let Ok(src) = c_str.to_str() else {
        return std::ptr::null_mut();
    };

    let tokens = Lexer::new(src).tokenize();
    let mut parser = Parser::new(tokens, src);
    let result = parser.parse();

    let doc = match result {
        Ok(value) => SpineDoc {
            root: Some(value),
            errors: Vec::new(),
        },
        Err(errors) => SpineDoc { root: None, errors },
    };

    Box::into_raw(Box::new(doc))
}

/// Parses Spine source with an associated filename for error messages.
///
/// # Safety
///
/// `input` must be a valid null-terminated C string.
/// `filename` must be a valid null-terminated C string, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_parse_named(
    input: *const c_char,
    filename: *const c_char,
) -> *mut SpineDoc {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };
    let Ok(src) = c_str.to_str() else {
        return std::ptr::null_mut();
    };

    let tokens = Lexer::new(src).tokenize();
    let mut parser = Parser::new(tokens, src);

    if !filename.is_null() {
        let fname_cstr = unsafe { CStr::from_ptr(filename) };
        if let Ok(fname) = fname_cstr.to_str() {
            parser = parser.with_source(fname);
        }
    }

    let result = parser.parse();

    let doc = match result {
        Ok(value) => SpineDoc {
            root: Some(value),
            errors: Vec::new(),
        },
        Err(errors) => SpineDoc { root: None, errors },
    };

    Box::into_raw(Box::new(doc))
}

/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn spine_has_errors(doc: *const SpineDoc) -> bool {
    if doc.is_null() {
        return true;
    }
    !unsafe { (*doc).errors.is_empty() }
}

/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_get_errors(doc: *const SpineDoc) -> *mut c_char {
    if doc.is_null() {
        return std::ptr::null_mut();
    }
    let errors = unsafe { (*doc).errors.join("\n\n") };
    CString::new(errors).map_or(std::ptr::null_mut(), CString::into_raw)
}

/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_doc_root(doc: *const SpineDoc) -> *const SpineValue {
    if doc.is_null() {
        return std::ptr::null();
    }
    unsafe { &(*doc).root }
        .as_ref()
        .map_or(std::ptr::null(), |value| {
            Box::into_raw(Box::new(SpineValue {
                inner: std::ptr::from_ref::<Value>(value),
            }))
        })
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn spine_value_type(val: *const SpineValue) -> c_int {
    if val.is_null() {
        return -1;
    }
    match unsafe { &*(*val).inner } {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(_) => 2,
        Value::String(_) => 3,
        Value::Array(_) => 4,
        Value::Object(_) => 5,
        Value::Tagged(_, _) => 6,
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn spine_value_bool(val: *const SpineValue) -> bool {
    if val.is_null() {
        return false;
    }
    match unsafe { &*(*val).inner } {
        Value::Bool(b) => *b,
        _ => false,
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn spine_value_number(val: *const SpineValue) -> c_double {
    if val.is_null() {
        return 0.0;
    }
    match unsafe { &*(*val).inner } {
        Value::Number(n) => *n,
        _ => 0.0,
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_value_string(val: *const SpineValue) -> *mut c_char {
    if val.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*(*val).inner } {
        Value::String(s) => {
            CString::new(s.as_str()).map_or(std::ptr::null_mut(), CString::into_raw)
        }
        _ => std::ptr::null_mut(),
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_value_tag(val: *const SpineValue) -> *mut c_char {
    if val.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*(*val).inner } {
        Value::Tagged(tag, _) => {
            CString::new(tag.as_str()).map_or(std::ptr::null_mut(), CString::into_raw)
        }
        _ => std::ptr::null_mut(),
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_value_tag_content(val: *const SpineValue) -> *mut c_char {
    if val.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*(*val).inner } {
        Value::Tagged(_, content) => {
            CString::new(content.as_str()).map_or(std::ptr::null_mut(), CString::into_raw)
        }
        _ => std::ptr::null_mut(),
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn spine_array_len(val: *const SpineValue) -> c_ulong {
    if val.is_null() {
        return 0;
    }
    match unsafe { &*(*val).inner } {
        Value::Array(arr) => arr.len() as c_ulong,
        _ => 0,
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_array_get(
    val: *const SpineValue,
    index: c_ulong,
) -> *const SpineValue {
    if val.is_null() {
        return std::ptr::null();
    }
    match unsafe { &*(*val).inner } {
        Value::Array(arr) => arr
            .get(usize::try_from(index).unwrap_or(usize::MAX))
            .map_or(std::ptr::null(), |v| {
                Box::into_raw(Box::new(SpineValue {
                    inner: std::ptr::from_ref::<Value>(v),
                }))
            }),
        _ => std::ptr::null(),
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn spine_object_len(val: *const SpineValue) -> c_ulong {
    if val.is_null() {
        return 0;
    }
    match unsafe { &*(*val).inner } {
        Value::Object(fields) => fields.len() as c_ulong,
        _ => 0,
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_object_key(val: *const SpineValue, index: c_ulong) -> *mut c_char {
    if val.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*(*val).inner } {
        Value::Object(fields) => fields
            .get(usize::try_from(index).unwrap_or(usize::MAX))
            .and_then(|(k, _)| CString::new(k.as_str()).ok())
            .map_or(std::ptr::null_mut(), CString::into_raw),
        _ => std::ptr::null_mut(),
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
/// `key` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_object_get(
    val: *const SpineValue,
    key: *const c_char,
) -> *const SpineValue {
    if val.is_null() || key.is_null() {
        return std::ptr::null();
    }
    let key_str = unsafe { CStr::from_ptr(key) };
    let Ok(key_str) = key_str.to_str() else {
        return std::ptr::null();
    };
    match unsafe { &*(*val).inner } {
        Value::Object(fields) => match fields.iter().find(|(k, _)| k == key_str) {
            Some((_, v)) => Box::into_raw(Box::new(SpineValue {
                inner: std::ptr::from_ref::<Value>(v),
            })),
            None => std::ptr::null(),
        },
        _ => std::ptr::null(),
    }
}

// --- //

/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_free_doc(doc: *mut SpineDoc) {
    if !doc.is_null() {
        unsafe {
            drop(Box::from_raw(doc));
        }
    }
}

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_free_value(val: *mut SpineValue) {
    if !val.is_null() {
        unsafe {
            drop(Box::from_raw(val));
        }
    }
}

/// # Safety
///
/// `s` must be a valid pointer to a C string allocated by Spine, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

/// Parser and spec metadata exposed by the ABI layer.
#[repr(C)]
pub struct SpineFormatDetails {
    pub version: *const c_char,
    pub spec: *const c_char,
    pub backend: *const c_char,
}

/// Returns the parser version, spec version, and whether this is native or WASM.
///
/// All pointers point to leaked static memory — no free is required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_format_details() -> SpineFormatDetails {
    fn leak(s: &str) -> *mut c_char {
        CString::new(s).unwrap().into_raw()
    }

    let backend = if cfg!(target_arch = "wasm32") {
        "wasm"
    } else {
        "native"
    };

    SpineFormatDetails {
        version: leak(env!("CARGO_PKG_VERSION")),
        spec: leak("1.0-rc.1"),
        backend: leak(backend),
    }
}

/// Frees the strings returned by `spine_format_details`.
///
/// # Safety
///
/// `details` must have been returned by `spine_format_details` and not freed already.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_free_format_details(details: SpineFormatDetails) {
    unsafe {
        drop(CString::from_raw(details.version as *mut c_char));
        drop(CString::from_raw(details.spec as *mut c_char));
        drop(CString::from_raw(details.backend as *mut c_char));
    }
}

/// Parse Spine source and return the AST as a JSON string.
///
/// The JSON output includes format metadata and either the parsed
/// value or error information:
///
/// ```json
/// {"version":"0.1.0","spec":"1.0-rc.1","backend":"native",ok":true,"value":{...}}
/// {"version":"0.1.0","spec":"1.0-rc.1","backend":"native","ok":false,"errors":[...]}
/// ```
///
/// The returned string must be freed with `spine_free_string`.
///
/// # Safety
///
/// `input` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spine_parse_json(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };
    let Ok(src) = c_str.to_str() else {
        return std::ptr::null_mut();
    };

    let tokens = Lexer::new(src).tokenize();
    let mut parser = Parser::new(tokens, src);
    let result = parser.parse();

    let backend = if cfg!(target_arch = "wasm32") {
        "wasm"
    } else {
        "native"
    };

    let mut json = String::with_capacity(4096);
    json.push('{');
    write_json_str("version", &mut json);
    json.push_str(":\"");
    json.push_str(env!("CARGO_PKG_VERSION"));
    json.push_str("\",");
    write_json_str("spec", &mut json);
    json.push_str(":\"1.0-rc.1\",");
        write_json_str("backend", &mut json);
        json.push_str(":\"");
        json.push_str(backend);
        json.push('"');

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
    CString::new(json).map_or(std::ptr::null_mut(), CString::into_raw)
}

fn write_json_str(s: &str, buf: &mut String) {
    buf.push('"');
    buf.push_str(s);
    buf.push('"');
}

fn write_json_value(val: &Value, buf: &mut String) {
    match val {
        Value::Null => buf.push_str("null"),
        Value::Bool(b) => buf.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => write_json_number(*n, buf),
        Value::String(s) => write_json_string(s, buf),
        Value::Tagged(tag, content) => {
            buf.push_str("{\"tag\":");
            write_json_string(tag, buf);
            buf.push_str(",\"content\":");
            write_json_string(content, buf);
            buf.push('}');
        }
        Value::Array(arr) => {
            buf.push('[');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    buf.push(',');
                }
                write_json_value(v, buf);
            }
            buf.push(']');
        }
        Value::Object(fields) => {
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
