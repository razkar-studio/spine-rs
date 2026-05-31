mod document;
mod ffi;
mod value;

pub use document::Document;
pub use value::{Value, ValueType};
use std::ffi::CStr;

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

#[cfg(test)]
mod tests;
