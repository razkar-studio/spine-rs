use std::os::raw::{c_char, c_double, c_int, c_ulong};

#[repr(C)]
pub struct SpineDoc {
    _private: [u8; 0],
}

#[repr(C)]
pub struct SpineValue {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn spine_parse(input: *const c_char) -> *mut SpineDoc;
    pub fn spine_parse_named(input: *const c_char, filename: *const c_char) -> *mut SpineDoc;
    pub fn spine_has_errors(doc: *const SpineDoc) -> bool;
    pub fn spine_get_errors(doc: *const SpineDoc) -> *mut c_char;
    pub fn spine_doc_root(doc: *const SpineDoc) -> *const SpineValue;
    pub fn spine_value_type(val: *const SpineValue) -> c_int;
    pub fn spine_value_bool(val: *const SpineValue) -> bool;
    pub fn spine_value_number(val: *const SpineValue) -> c_double;
    pub fn spine_value_string(val: *const SpineValue) -> *mut c_char;
    pub fn spine_value_tag(val: *const SpineValue) -> *mut c_char;
    pub fn spine_value_tag_content(val: *const SpineValue) -> *mut c_char;
    pub fn spine_array_len(val: *const SpineValue) -> c_ulong;
    pub fn spine_array_get(val: *const SpineValue, index: c_ulong) -> *const SpineValue;
    pub fn spine_object_len(val: *const SpineValue) -> c_ulong;
    pub fn spine_object_key(val: *const SpineValue, index: c_ulong) -> *mut c_char;
    pub fn spine_object_get(val: *const SpineValue, key: *const c_char) -> *const SpineValue;
    pub fn spine_free_doc(doc: *mut SpineDoc);
    pub fn spine_free_value(val: *mut SpineValue);
    pub fn spine_free_string(s: *mut c_char);
}
