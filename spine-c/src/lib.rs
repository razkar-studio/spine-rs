use spine_rs::{Lexer, Parser, Value};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_ulong};

/// Opaque types that C sees as pointers
pub struct SpineDoc {
    root: Option<Value>,
    errors: Vec<String>,
}

pub struct SpineValue {
    inner: *const Value,
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_parse(input: *const c_char) -> *mut SpineDoc {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };
    let src = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
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

#[unsafe(no_mangle)]
pub const extern "C" fn spine_has_errors(doc: *const SpineDoc) -> bool {
    if doc.is_null() {
        return true;
    }
    !unsafe { (*doc).errors.is_empty() }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_get_errors(doc: *const SpineDoc) -> *mut c_char {
    if doc.is_null() {
        return std::ptr::null_mut();
    }
    let errors = unsafe { (*doc).errors.join("\n") };
    match CString::new(errors) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_doc_root(doc: *const SpineDoc) -> *const SpineValue {
    if doc.is_null() {
        return std::ptr::null();
    }
    match unsafe { &(*doc).root } {
        Some(value) => Box::into_raw(Box::new(SpineValue {
            inner: std::ptr::from_ref::<Value>(value),
        })),
        None => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub const extern "C" fn spine_value_type(val: *const SpineValue) -> c_int {
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

#[unsafe(no_mangle)]
pub const extern "C" fn spine_value_bool(val: *const SpineValue) -> bool {
    if val.is_null() {
        return false;
    }
    match unsafe { &*(*val).inner } {
        Value::Bool(b) => *b,
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub const extern "C" fn spine_value_number(val: *const SpineValue) -> c_double {
    if val.is_null() {
        return 0.0;
    }
    match unsafe { &*(*val).inner } {
        Value::Number(n) => *n,
        _ => 0.0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_value_string(val: *const SpineValue) -> *mut c_char {
    if val.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*(*val).inner } {
        Value::String(s) => match CString::new(s.as_str()) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        _ => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_value_tag(val: *const SpineValue) -> *mut c_char {
    if val.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*(*val).inner } {
        Value::Tagged(tag, _) => match CString::new(tag.as_str()) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        _ => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_value_tag_content(val: *const SpineValue) -> *mut c_char {
    if val.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*(*val).inner } {
        Value::Tagged(_, content) => match CString::new(content.as_str()) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        _ => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub const extern "C" fn spine_array_len(val: *const SpineValue) -> c_ulong {
    if val.is_null() {
        return 0;
    }
    match unsafe { &*(*val).inner } {
        Value::Array(arr) => arr.len() as c_ulong,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_array_get(val: *const SpineValue, index: c_ulong) -> *const SpineValue {
    if val.is_null() {
        return std::ptr::null();
    }
    match unsafe { &*(*val).inner } {
        Value::Array(arr) => match arr.get(index as usize) {
            Some(v) => Box::into_raw(Box::new(SpineValue {
                inner: std::ptr::from_ref::<Value>(v),
            })),
            None => std::ptr::null(),
        },
        _ => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub const extern "C" fn spine_object_len(val: *const SpineValue) -> c_ulong {
    if val.is_null() {
        return 0;
    }
    match unsafe { &*(*val).inner } {
        Value::Object(fields) => fields.len() as c_ulong,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_object_key(val: *const SpineValue, index: c_ulong) -> *mut c_char {
    if val.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*(*val).inner } {
        Value::Object(fields) => match fields.get(index as usize) {
            Some((k, _)) => match CString::new(k.as_str()) {
                Ok(cs) => cs.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            None => std::ptr::null_mut(),
        },
        _ => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_object_get(
    val: *const SpineValue,
    key: *const c_char,
) -> *const SpineValue {
    if val.is_null() || key.is_null() {
        return std::ptr::null();
    }
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
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

#[unsafe(no_mangle)]
pub extern "C" fn spine_free_doc(doc: *mut SpineDoc) {
    if !doc.is_null() {
        unsafe {
            drop(Box::from_raw(doc));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_free_value(val: *mut SpineValue) {
    if !val.is_null() {
        unsafe {
            drop(Box::from_raw(val));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn spine_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}
