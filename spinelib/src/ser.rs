#[derive(Debug)]
pub enum SerError {
    Custom(String),
}

impl serde::ser::Error for SerError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SerError::Custom(msg.to_string())
    }
}

impl std::error::Error for SerError {}

impl std::fmt::Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SerError::Custom(s) => write!(f, "{s}"),
        }
    }
}

pub struct SeqSerializer {
    items: Vec<spine_rs::Value>,
}

impl serde::ser::SerializeSeq for SeqSerializer {
    type Ok = spine_rs::Value;
    type Error = SerError;

    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), SerError> {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<spine_rs::Value, SerError> {
        Ok(spine_rs::Value::Array(self.items))
    }
}

macro_rules! impl_seq_serializer {
    ($trait:ident, $method:ident) => {
        impl serde::ser::$trait for SeqSerializer {
            type Ok = spine_rs::Value;
            type Error = SerError;

            fn $method<T: serde::Serialize + ?Sized>(&mut self, value: &T) -> Result<(), SerError> {
                self.items.push(value.serialize(Serializer)?);
                Ok(())
            }

            fn end(self) -> Result<spine_rs::Value, SerError> {
                Ok(spine_rs::Value::Array(self.items))
            }
        }
    };
}

impl_seq_serializer!(SerializeTuple, serialize_element);
impl_seq_serializer!(SerializeTupleStruct, serialize_field);
impl_seq_serializer!(SerializeTupleVariant, serialize_field);

pub struct MapSerializer {
    fields: Vec<(String, spine_rs::Value)>,
    current_key: Option<String>,
}

macro_rules! impl_map_serializer {
    ($trait:ident) => {
        impl serde::ser::$trait for MapSerializer {
            type Ok = spine_rs::Value;
            type Error = SerError;

            fn serialize_field<T: serde::Serialize + ?Sized>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), SerError> {
                self.fields
                    .push((key.to_string(), value.serialize(Serializer)?));
                Ok(())
            }

            fn end(self) -> Result<spine_rs::Value, SerError> {
                Ok(spine_rs::Value::Object(self.fields))
            }
        }
    };
}

impl_map_serializer!(SerializeStruct);
impl_map_serializer!(SerializeStructVariant);

impl serde::ser::SerializeMap for MapSerializer {
    type Ok = spine_rs::Value;
    type Error = SerError;

    fn serialize_key<T: serde::Serialize + ?Sized>(&mut self, key: &T) -> Result<(), SerError> {
        let k = key.serialize(Serializer)?;
        match k {
            spine_rs::Value::String(s) => {
                self.current_key = Some(s);
                Ok(())
            }
            _ => Err(SerError::Custom("map keys must be strings".into())),
        }
    }

    fn serialize_value<T: serde::Serialize + ?Sized>(&mut self, value: &T) -> Result<(), SerError> {
        let key = self
            .current_key
            .take()
            .ok_or_else(|| SerError::Custom("value before key".into()))?;
        self.fields.push((key, value.serialize(Serializer)?));
        Ok(())
    }

    fn end(self) -> Result<spine_rs::Value, SerError> {
        Ok(spine_rs::Value::Object(self.fields))
    }
}

pub struct Serializer;

pub fn to_document<T: serde::Serialize>(value: &T) -> Result<crate::Document, SerError> {
    let value = value.serialize(Serializer)?;
    Ok(crate::Document::from_value(value))
}

impl serde::Serializer for Serializer {
    type Ok = spine_rs::Value;
    type Error = SerError;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = SeqSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = MapSerializer;

    fn serialize_bool(self, v: bool) -> Result<spine_rs::Value, SerError> {
        Ok(spine_rs::Value::Bool(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<spine_rs::Value, SerError> {
        Ok(spine_rs::Value::Number(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Number(v as f64))
    }

    fn serialize_str(self, v: &str) -> Result<spine_rs::Value, SerError> {
        Ok(spine_rs::Value::String(v.to_string()))
    }

    fn serialize_none(self) -> Result<spine_rs::Value, SerError> {
        Ok(spine_rs::Value::Null)
    }

    fn serialize_some<T: serde::Serialize + ?Sized>(
        self,
        value: &T,
    ) -> Result<spine_rs::Value, SerError> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<spine_rs::Value, SerError> {
        Ok(spine_rs::Value::Null)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::String(v.to_string()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(SerError::Custom("bytes not supported in Spine".into()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(spine_rs::Value::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let mut map = vec![];
        map.push((variant.to_string(), value.serialize(Serializer)?));
        Ok(spine_rs::Value::Object(map))
    }

    // --- //

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqSerializer { items: vec![] })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer {
            fields: vec![],
            current_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }
}
