use crate::value::Value;
use spine_rs::{Lexer, Parser};
use std::path::PathBuf;

/// A parsed Spine document.
///
/// Wraps the root `spine_rs::Value` and provides constructors that
/// handle the full lex-and-parse pipeline.
pub struct Document {
    root: Option<spine_rs::Value>,
}

/// Errors that can occur when loading a Spine document.
#[derive(Debug)]
pub enum DocError {
    /// An I/O error (file not found, permission denied, etc.).
    Io(std::io::Error),
    /// One or more parse errors.
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

    /// Creates a document from a pre-parsed `spine_rs::Value`.
    #[must_use]
    pub fn from_value(value: spine_rs::Value) -> Self {
        Self { root: Some(value) }
    }

    /// Serializes the document back to a Spine string.
    ///
    /// Returns `None` if the document has no root value.
    #[must_use]
    pub fn to_string(&self) -> Option<String> {
        self.root
            .as_ref()
            .map(crate::writer::to_string_inner)
    }

    /// Parses a Spine string, panicking on parse errors.
    ///
    /// This is a convenience method for use in contexts where parse
    /// errors should terminate the process (e.g. configuration loading
    /// at startup).
    ///
    /// # Panics
    ///
    /// Panics if the input contains lexical or parse errors.
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

    /// Parses a Spine file from the filesystem.
    ///
    /// # Errors
    ///
    /// Returns `DocError::Io` if the file cannot be read, or
    /// `DocError::Parse` if the content is not valid Spine.
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self, DocError> {
        let path = path.into();
        let contents = std::fs::read_to_string(&path)?;
        let tokens = Lexer::new(&contents).tokenize();
        let mut parser = Parser::new(tokens, &contents).with_source(&path.to_string_lossy());
        Self::from_parse_result(parser.parse())
    }

    /// Parses a Spine string into a document.
    ///
    /// # Errors
    ///
    /// Returns `DocError::Parse` if the input is not valid Spine.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: impl Into<String>) -> Result<Self, DocError> {
        let input = input.into();
        let tokens = Lexer::new(&input).tokenize();
        let mut parser = Parser::new(tokens, &input);
        Self::from_parse_result(parser.parse())
    }

    /// Returns a reference to the root `Value`, if present.
    ///
    /// An empty document (no statements) has no root value.
    #[must_use]
    pub fn root(&self) -> Option<Value> {
        self.root.as_ref().map(|v| Value::from_inner(v.clone()))
    }
}
