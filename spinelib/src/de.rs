use serde::Deserialize;

use serde::de::{self, MapAccess, SeqAccess, Visitor};

/// Errors that can occur during Spine deserialization.
#[derive(Debug)]
pub enum DeError {
    /// A custom error message.
    Custom(String),
}

impl std::fmt::Display for DeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeError::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for DeError {}

impl de::Error for DeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DeError::Custom(msg.to_string())
    }
}

/// Deserializer for Spine values.
pub struct Deserializer {
    value: spine_rs::Value,
}

impl Deserializer {
    /// Creates a new deserializer from a `spine_rs::Value`.
    #[must_use]
    pub fn from_value(value: spine_rs::Value) -> Self {
        Self { value }
    }
}

/// Deserializes a Spine `Document` into a Rust type.
///
/// # Errors
///
/// Returns `DeError` if the document is empty or the value cannot be
/// deserialized into the target type.
pub fn from_document<T: for<'de> Deserialize<'de>>(doc: &crate::Document) -> Result<T, DeError> {
    let value = doc.root().ok_or(DeError::Custom("empty document".into()))?;
    let de = Deserializer::from_value(value.into_inner());
    T::deserialize(de)
}

impl<'de> serde::Deserializer<'de> for Deserializer {
    type Error = DeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, DeError> {
        match self.value {
            spine_rs::Value::Null => visitor.visit_unit(),
            spine_rs::Value::Bool(b) => visitor.visit_bool(b),
            spine_rs::Value::Number(n) => visitor.visit_f64(n),
            spine_rs::Value::String(s) => visitor.visit_string(s),
            spine_rs::Value::Tagged(_, content) => visitor.visit_string(content),
            spine_rs::Value::Array(arr) => visitor.visit_seq(SeqDeserializer::new(arr)),
            spine_rs::Value::Object(fields) => visitor.visit_map(MapDeserializer::new(fields)),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct MapDeserializer {
    items: std::vec::IntoIter<(String, spine_rs::Value)>,
    current_value: Option<spine_rs::Value>,
}

impl MapDeserializer {
    fn new(fields: Vec<(String, spine_rs::Value)>) -> Self {
        Self {
            items: fields.into_iter(),
            current_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = DeError;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, DeError> {
        match self.items.next() {
            Some((key, value)) => {
                self.current_value = Some(value);
                seed.deserialize(serde::de::value::StringDeserializer::new(key))
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, DeError> {
        let value = self
            .current_value
            .take()
            .ok_or_else(|| DeError::Custom("value called before key".into()))?;
        seed.deserialize(Deserializer::from_value(value))
    }
}

struct SeqDeserializer {
    items: std::vec::IntoIter<spine_rs::Value>,
}

impl SeqDeserializer {
    fn new(arr: Vec<spine_rs::Value>) -> Self {
        Self {
            items: arr.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = DeError;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, DeError> {
        match self.items.next() {
            Some(value) => seed.deserialize(Deserializer::from_value(value)).map(Some),
            None => Ok(None),
        }
    }
}
