use crate::ffi;
use crate::value::Value;
use std::{
    ffi::{CStr, CString},
    fs,
    path::PathBuf,
};

pub struct Document {
    ptr: *mut ffi::SpineDoc,
}

#[derive(Debug)]
pub enum DocError {
    Io(std::io::Error),
    Parse(Vec<String>),
}

impl From<std::io::Error> for DocError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<Vec<String>> for DocError {
    fn from(e: Vec<String>) -> Self {
        Self::Parse(e)
    }
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Document")
    }
}

impl Document {
    #[must_use]
    pub fn from_str_or_panic(input: impl Into<String>) -> Self {
        Self::from_str(input).unwrap_or_else(|errors| {
            match &errors {
                DocError::Parse(errs) => {
                    for e in errs {
                        println!("{e}");
                    }
                }
                DocError::Io(e) => {
                    println!("{e}");
                }
            }
            std::process::exit(1)
        })
    }

    /// Loads a Spine document from a file path.
    ///
    /// # Errors
    ///
    /// Returns a `DocError` if the file cannot be read or the content is invalid.
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self, DocError> {
        let path = path.into();
        let contents = fs::read_to_string(path)?;
        Self::from_str(contents)
    }

    /// Parses a Spine document from a string.
    ///
    /// # Errors
    ///
    /// Returns a `DocError` if the string contains a null byte or the content is invalid.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: impl Into<String>) -> Result<Self, DocError> {
        let input = input.into();
        let c_input = CString::new(input).map_err(|_| vec!["invalid input string".to_string()])?;
        let ptr = unsafe { ffi::spine_parse(c_input.as_ptr()) };

        if ptr.is_null() {
            return Err(DocError::Parse(vec![
                "spine_parse returned null".to_string(),
            ]));
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
                return Err(DocError::Parse(vec![s]));
            };
            return Err(DocError::Parse(errors));
        }

        Ok(doc)
    }

    #[must_use]
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
