/// The Spine parser.
///
/// This crate provides a spec-compliant lexer and parser for the Spine
/// configuration language. It produces an AST of `Value` nodes from
/// Spine source text.
pub mod value;
pub use value::Value;

/// Token types and spanned-token alias.
pub mod token;
pub use token::{SpannedToken, Token};

/// Lexer: tokenizes Spine source text.
pub mod lexer;
pub use lexer::Lexer;

/// Parser: builds a `Value` tree from a token stream.
pub mod parser;
pub use parser::Parser;

#[cfg(test)]
mod tests;
