#[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Tagged(String, String),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}
