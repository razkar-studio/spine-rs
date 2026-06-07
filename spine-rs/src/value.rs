/// A parsed Spine value.
///
/// Represents any value that can appear in a Spine document: scalars,
/// arrays, objects, and tagged literals.
#[derive(Debug, Default, PartialEq, Clone)]
pub enum Value {
    /// The absence of a value.
    #[default]
    Null,
    /// A boolean (`true` or `false`).
    Bool(bool),
    /// A 64-bit floating-point number.
    Number(f64),
    /// A UTF-8 string.
    String(String),
    /// An ordered list of values.
    Array(Vec<Self>),
    /// A mapping of string keys to values, preserving insertion order.
    Object(Vec<(String, Self)>),
    /// A tagged literal consisting of a tag name and string content.
    Tagged(String, String),
}
