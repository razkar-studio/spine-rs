#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Pipe,
    Equals,
    Tilde,
    Dash,
    Dot,
    Newline,
    Ident(String),
    Str(String),
    Number(f64),
    Bool(bool),
    Null,
    LineComment(String),
    BlockComment(String),
    Tagged(String, String),
}
