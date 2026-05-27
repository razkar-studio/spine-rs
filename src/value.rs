pub enum Value {
    Null,
    Bool(bool),
    Number(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Tagged(String, String),
}
