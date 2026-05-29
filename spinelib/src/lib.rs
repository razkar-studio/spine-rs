mod document;
mod ffi;
mod value;

pub use document::Document;
pub use value::{Value, ValueType};

#[cfg(test)]
mod tests;
