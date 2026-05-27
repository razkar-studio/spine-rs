use crate::ffi;
use std::ffi::{CStr, CString};

pub struct Value {
    ptr: *const ffi::SpineValue,
}

impl Value {
    pub(crate) fn from_ptr(ptr: *const ffi::SpineValue) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        Some(Self { ptr })
    }

    pub fn value_type(&self) -> ValueType {
        match unsafe { ffi::spine_value_type(self.ptr) } {
            0 => ValueType::Null,
            1 => ValueType::Bool,
            2 => ValueType::Number,
            3 => ValueType::String,
            4 => ValueType::Array,
            5 => ValueType::Object,
            6 => ValueType::Tagged,
            _ => ValueType::Null,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.value_type() {
            ValueType::Bool => Some(unsafe { ffi::spine_value_bool(self.ptr) }),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.value_type() {
            ValueType::Number => Some(unsafe { ffi::spine_value_number(self.ptr) }),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<String> {
        match self.value_type() {
            ValueType::String => {
                let ptr = unsafe { ffi::spine_value_string(self.ptr) };
                if ptr.is_null() {
                    return None;
                }
                let s = unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned();
                unsafe {
                    ffi::spine_free_string(ptr);
                }
                Some(s)
            }
            _ => None,
        }
    }

    pub fn tag(&self) -> Option<(String, String)> {
        match self.value_type() {
            ValueType::Tagged => {
                let tag_ptr = unsafe { ffi::spine_value_tag(self.ptr) };
                let content_ptr = unsafe { ffi::spine_value_tag_content(self.ptr) };
                if tag_ptr.is_null() || content_ptr.is_null() {
                    return None;
                }
                let tag = unsafe { CStr::from_ptr(tag_ptr) }
                    .to_string_lossy()
                    .into_owned();
                let content = unsafe { CStr::from_ptr(content_ptr) }
                    .to_string_lossy()
                    .into_owned();
                unsafe {
                    ffi::spine_free_string(tag_ptr);
                }
                unsafe {
                    ffi::spine_free_string(content_ptr);
                }
                Some((tag, content))
            }
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match self.value_type() {
            ValueType::Array => unsafe { ffi::spine_array_len(self.ptr) as usize },
            ValueType::Object => unsafe { ffi::spine_object_len(self.ptr) as usize },
            _ => 0,
        }
    }

    pub fn get_index(&self, index: usize) -> Option<Value> {
        match self.value_type() {
            ValueType::Array => {
                let ptr = unsafe { ffi::spine_array_get(self.ptr, index as u64) };
                Value::from_ptr(ptr)
            }
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        match self.value_type() {
            ValueType::Object => {
                let key = CString::new(key).ok()?;
                let ptr = unsafe { ffi::spine_object_get(self.ptr, key.as_ptr()) };
                Value::from_ptr(ptr)
            }
            _ => None,
        }
    }

    pub fn key_at(&self, index: usize) -> Option<String> {
        match self.value_type() {
            ValueType::Object => {
                let ptr = unsafe { ffi::spine_object_key(self.ptr, index as u64) };
                if ptr.is_null() {
                    return None;
                }
                let s = unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned();
                unsafe {
                    ffi::spine_free_string(ptr);
                }
                Some(s)
            }
            _ => None,
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            ffi::spine_free_value(self.ptr as *mut ffi::SpineValue);
        }
    }
}

pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
    Tagged,
}
