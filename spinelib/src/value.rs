/// A Spine value with ergonomic accessor methods.
///
/// Wraps `spine_rs::Value` and provides typed accessors for inspecting
/// the AST without pattern-matching on the core enum directly.
pub struct Value {
    inner: spine_rs::Value,
}

impl Value {
    pub(crate) fn from_inner(inner: spine_rs::Value) -> Self {
        Self { inner }
    }

    pub(crate) fn into_inner(self) -> spine_rs::Value {
        self.inner
    }

    /// Returns the variant type of this value.
    #[must_use]
    pub fn value_type(&self) -> ValueType {
        match self.inner {
            spine_rs::Value::Null => ValueType::Null,
            spine_rs::Value::Bool(_) => ValueType::Bool,
            spine_rs::Value::Number(_) => ValueType::Number,
            spine_rs::Value::String(_) => ValueType::String,
            spine_rs::Value::Array(_) => ValueType::Array,
            spine_rs::Value::Object(_) => ValueType::Object,
            spine_rs::Value::Tagged(_, _) => ValueType::Tagged,
        }
    }

    /// Returns the boolean value if this is a `Bool`, otherwise `None`.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match &self.inner {
            spine_rs::Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the numeric value if this is a `Number`, otherwise `None`.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match &self.inner {
            spine_rs::Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the string value if this is a `String`, otherwise `None`.
    #[must_use]
    pub fn as_str(&self) -> Option<String> {
        match &self.inner {
            spine_rs::Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Returns the `(tag, content)` pair if this is a `Tagged`, otherwise `None`.
    #[must_use]
    pub fn tag(&self) -> Option<(String, String)> {
        match &self.inner {
            spine_rs::Value::Tagged(tag, content) => Some((tag.clone(), content.clone())),
            _ => None,
        }
    }

    /// Returns `true` if this value is an empty array or empty object.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements for arrays and objects, or `0` for
    /// all other types.
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.inner {
            spine_rs::Value::Array(arr) => arr.len(),
            spine_rs::Value::Object(fields) => fields.len(),
            _ => 0,
        }
    }

    /// Returns the element at `index` if this is an array.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<Self> {
        match &self.inner {
            spine_rs::Value::Array(arr) => arr.get(index).map(|v| Self::from_inner(v.clone())),
            _ => None,
        }
    }

    /// Returns the value at `key` if this is an object.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Self> {
        match &self.inner {
            spine_rs::Value::Object(fields) => fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| Self::from_inner(v.clone())),
            _ => None,
        }
    }

    /// Returns the key at `index` if this is an object.
    #[must_use]
    pub fn key_at(&self, index: usize) -> Option<String> {
        match &self.inner {
            spine_rs::Value::Object(fields) => fields.get(index).map(|(k, _)| k.clone()),
            _ => None,
        }
    }
}

/// The type of a Spine value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    /// The `null` value.
    Null,
    /// A boolean (`true` or `false`).
    Bool,
    /// A numeric literal.
    Number,
    /// A string literal.
    String,
    /// An ordered list of values.
    Array,
    /// A key-value mapping.
    Object,
    /// A tagged literal.
    Tagged,
}
