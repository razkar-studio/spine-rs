mod document;
mod ffi;
mod value;

pub use document::Document;
pub use value::{Value, ValueType};
use std::ffi::{CStr, CString};

/// Build-time metadata about the ABI library linked at runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatDetails {
    /// The parser version (from `spine-abi`'s Cargo.toml).
    pub version: String,
    /// The spec version this parser targets.
    pub spec: String,
    /// Whether the linked library is native or WASM.
    pub backend: String,
}

/// Returns metadata about the linked ABI library (`spine-abi`).
///
/// This can be used to determine the parser version, the spec
/// version, and whether the backend is native or running in a
/// WASM runtime.
pub fn format_details() -> FormatDetails {
    let raw = unsafe { ffi::spine_format_details() };
    let details = FormatDetails {
        version: unsafe { CStr::from_ptr(raw.version) }.to_string_lossy().into(),
        spec: unsafe { CStr::from_ptr(raw.spec) }.to_string_lossy().into(),
        backend: unsafe { CStr::from_ptr(raw.backend) }.to_string_lossy().into(),
    };
    unsafe { ffi::spine_free_format_details(raw) };
    details
}

/// Parse Spine source and return the AST as a JSON string.
///
/// This is a convenience wrapper around `spine_parse_json` from the ABI.
/// The JSON output includes format metadata, success status, and either
/// the parsed AST or a list of errors.
///
/// Useful for environments where navigating the pointer-based object API
/// is cumbersome (e.g., WASM, scripting languages).
pub fn parse_to_json(input: &str) -> String {
    let c_input = CString::new(input).unwrap_or_default();
    let ptr = unsafe { ffi::spine_parse_json(c_input.as_ptr()) };
    if ptr.is_null() {
        return String::new();
    }
    let result = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into();
    unsafe { ffi::spine_free_string(ptr) };
    result
}

#[cfg(test)]
mod tests;
