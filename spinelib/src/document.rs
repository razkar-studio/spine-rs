use crate::ffi;
use crate::value::Value;
use std::ffi::{CStr, CString};

pub struct Document {
    ptr: *mut ffi::SpineDoc,
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Document")
    }
}

impl Document {
    pub fn parse(input: &str) -> Result<Self, Vec<String>> {
        let c_input = CString::new(input).map_err(|_| vec!["invalid input string".to_string()])?;
        let ptr = unsafe { ffi::spine_parse(c_input.as_ptr()) };

        if ptr.is_null() {
            return Err(vec!["spine_parse returned null".to_string()]);
        }

        let doc = Self { ptr };

        if unsafe { ffi::spine_has_errors(ptr) } {
            let err_ptr = unsafe { ffi::spine_get_errors(ptr) };
            let errors = if err_ptr.is_null() {
                vec!["unknown parse error".to_string()]
            } else {
                let s = unsafe { CStr::from_ptr(err_ptr) }
                    .to_string_lossy()
                    .into_owned();
                unsafe {
                    ffi::spine_free_string(err_ptr);
                }
                return Err(vec![s]);
            };
            return Err(errors);
        }

        Ok(doc)
    }

    pub fn root(&self) -> Option<Value> {
        let ptr = unsafe { ffi::spine_doc_root(self.ptr) };
        Value::from_ptr(ptr)
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        unsafe {
            ffi::spine_free_doc(self.ptr);
        }
    }
}
