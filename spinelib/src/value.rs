pub struct Value {
    inner: spine_rs::Value,
}

impl Value {
    pub(crate) fn from_inner(inner: spine_rs::Value) -> Self {
        Self { inner }
    }

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

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match &self.inner {
            spine_rs::Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match &self.inner {
            spine_rs::Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> Option<String> {
        match &self.inner {
            spine_rs::Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn tag(&self) -> Option<(String, String)> {
        match &self.inner {
            spine_rs::Value::Tagged(tag, content) => Some((tag.clone(), content.clone())),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match &self.inner {
            spine_rs::Value::Array(arr) => arr.len(),
            spine_rs::Value::Object(fields) => fields.len(),
            _ => 0,
        }
    }

    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<Self> {
        match &self.inner {
            spine_rs::Value::Array(arr) => {
                arr.get(index).map(|v| Self::from_inner(v.clone()))
            }
            _ => None,
        }
    }

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

    #[must_use]
    pub fn key_at(&self, index: usize) -> Option<String> {
        match &self.inner {
            spine_rs::Value::Object(fields) => {
                fields.get(index).map(|(k, _)| k.clone())
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
    Tagged,
}
