use crate::value::Value;
use spine_rs::{Lexer, Parser};
use std::path::PathBuf;

pub struct Document {
    root: Option<spine_rs::Value>,
}

#[derive(Debug)]
pub enum DocError {
    Io(std::io::Error),
    Parse(Vec<String>),
}

impl From<std::io::Error> for DocError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<Vec<String>> for DocError {
    fn from(e: Vec<String>) -> Self {
        Self::Parse(e)
    }
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Document")
    }
}

impl Document {
    fn from_parse_result(result: Result<spine_rs::Value, Vec<String>>) -> Result<Self, DocError> {
        match result {
            Ok(value) => Ok(Document { root: Some(value) }),
            Err(errors) => Err(DocError::Parse(errors)),
        }
    }

    pub fn from_value(value: spine_rs::Value) -> Self {
        Self { root: Some(value) }
    }

    #[must_use]
    pub fn from_str_or_panic(input: impl Into<String>) -> Self {
        Self::from_str(input).unwrap_or_else(|errors| {
            match &errors {
                DocError::Parse(errs) => {
                    for e in errs {
                        println!("{e}");
                    }
                }
                DocError::Io(e) => {
                    println!("{e}");
                }
            }
            std::process::exit(1)
        })
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self, DocError> {
        let path = path.into();
        let contents = std::fs::read_to_string(&path)?;
        let tokens = Lexer::new(&contents).tokenize();
        let mut parser = Parser::new(tokens, &contents).with_source(&path.to_string_lossy());
        Self::from_parse_result(parser.parse())
    }

    pub fn from_str(input: impl Into<String>) -> Result<Self, DocError> {
        let input = input.into();
        let tokens = Lexer::new(&input).tokenize();
        let mut parser = Parser::new(tokens, &input);
        Self::from_parse_result(parser.parse())
    }

    #[must_use]
    pub fn root(&self) -> Option<Value> {
        self.root.as_ref().map(|v| Value::from_inner(v.clone()))
    }
}
