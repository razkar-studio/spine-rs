#[derive(Debug, Default, PartialEq, Clone)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Self>),
    Object(Vec<(String, Self)>),
    Tagged(String, String),
}
