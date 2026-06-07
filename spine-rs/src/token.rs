/// A lexical token produced by the Spine lexer.
///
/// Each token represents a single syntactic element from the source text.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// The `|` character used for indentation.
    Pipe,
    /// The `=` character used in key-value assignments.
    Equals,
    /// The `~` character used for append operations.
    Tilde,
    /// The `-` character used for array elements.
    Dash,
    /// The `.` character used in dotted key paths.
    Dot,
    /// A newline (`\n`), which terminates a statement.
    Newline,
    /// An identifier (key name, bare value before `=`).
    Ident(String),
    /// A quoted string literal, including multiline strings.
    Str(String),
    /// A numeric literal parsed as a 64-bit float.
    Number(f64),
    /// A boolean keyword (`true` or `false`).
    Bool(bool),
    /// The `null` keyword.
    Null,
    /// A line comment starting with `#`.
    LineComment(String),
    /// A block comment delimited by `/*` and `*/`.
    BlockComment(String),
    /// A tagged literal (e.g. `date"2026-01-01"`).
    Tagged(String, String),
    /// A character that has no meaning in Spine syntax.
    Unknown(char),
    /// A lexical error message produced during tokenization.
    Error(String),
}

/// A token annotated with its source location.
///
/// The tuple is `(token, line, column)`, where line and column are
/// 1-indexed positions in the source text.
pub type SpannedToken = (Token, usize, usize);
