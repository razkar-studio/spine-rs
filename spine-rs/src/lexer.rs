use crate::{SpannedToken, Token};

/// Tokenizes Spine source text into a stream of `SpannedToken` values.
///
/// The lexer handles indentation tracking (pipe counts), string and
/// multiline string lexing, escape sequences, tagged literals, comments,
/// and bare-value heuristics (number, bool, null inference).
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line_pipes: usize,
    line: usize,
    col: usize,
    after_value_start: bool,
}

impl Lexer {
    /// Creates a new lexer for the given source text.
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line_pipes: 0,
            line: 1,
            col: 1,
            after_value_start: false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.get(self.pos).copied();
        self.pos += 1;
        match c {
            Some('\n') => {
                self.line += 1;
                self.col = 1;
            }
            Some(_) => {
                self.col += 1;
            }
            None => {}
        }
        c
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn skip_line_comment(&mut self) -> Token {
        self.advance();
        let mut content = String::new();
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            content.push(c);
            self.advance();
        }
        Token::LineComment(content.trim().to_string())
    }

    fn skip_block_comment(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        self.advance();
        self.advance();
        let mut content = String::new();
        let mut depth = 1;
        while !self.is_at_end() {
            match (self.peek(), self.input.get(self.pos + 1).copied()) {
                (Some('/'), Some('*')) => {
                    depth += 1;
                    content.push_str("/*");
                    self.advance();
                    self.advance();
                }
                (Some('*'), Some('/')) => {
                    depth -= 1;
                    self.advance();
                    self.advance();
                    if depth == 0 {
                        return Token::BlockComment(content.trim().to_string());
                    }
                    content.push_str("*/");
                }
                #[allow(unused_assignments)]
                (Some(c), _) => {
                    content.push(c);
                    self.advance();
                }
                _ => break,
            }
        }
        Token::Error(format!(
            "{start_line}:{start_col} unterminated block comment"
        ))
    }

    #[allow(clippy::too_many_lines)]
    fn lex_string(&mut self) -> Token {
        self.after_value_start = false;
        let start_line = self.line;
        let start_col = self.col;
        self.advance();

        if self.peek() == Some('"') {
            self.advance();
            if self.peek() == Some('"') {
                self.advance();
                return self.lex_multiline_string();
            }
            return Token::Str(String::new());
        }

        let mut s = String::new();
        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.advance();
                    return Token::Str(s);
                }
                '\n' => {
                    return Token::Error(format!("{start_line}:{start_col} unterminated string"));
                }
                '\\' => {
                    self.advance();
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some('0') => s.push('\0'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        Some('x') => {
                            let (d1, d2) = (
                                self.input.get(self.pos).copied(),
                                self.input.get(self.pos + 1).copied(),
                            );
                            match (d1, d2) {
                                (Some(h1), Some(h2))
                                    if h1.is_ascii_hexdigit() && h2.is_ascii_hexdigit() =>
                                {
                                    self.advance();
                                    self.advance();
                                    let val =
                                        (h1.to_digit(16).unwrap() << 4) | h2.to_digit(16).unwrap();
                                    s.push(char::from_u32(val).unwrap());
                                }
                                _ => {
                                    return Token::Error(format!(
                                        "{start_line}:{start_col} invalid \\x escape"
                                    ));
                                }
                            }
                        }
                        Some('u') => {
                            if self.peek() == Some('{') {
                                self.advance();
                                let mut hex = String::new();
                                loop {
                                    match self.peek() {
                                        Some('}') => {
                                            self.advance();
                                            break;
                                        }
                                        Some(c) if c.is_ascii_hexdigit() => {
                                            hex.push(c);
                                            self.advance();
                                        }
                                        _ => {
                                            return Token::Error(format!(
                                                "{start_line}:{start_col} invalid \\u{{}} escape"
                                            ));
                                        }
                                    }
                                }
                                if hex.is_empty() {
                                    return Token::Error(format!(
                                        "{start_line}:{start_col} empty \\u{{}} escape"
                                    ));
                                }
                                let val = u32::from_str_radix(&hex, 16).unwrap();
                                if val > 0x0010_FFFF {
                                    return Token::Error(format!(
                                        "{start_line}:{start_col} unicode escape out of range"
                                    ));
                                }
                                if (0xD800..=0xDFFF).contains(&val) {
                                    return Token::Error(format!(
                                        "{start_line}:{start_col} surrogate unicode escape"
                                    ));
                                }
                                s.push(char::from_u32(val).unwrap());
                            } else {
                                let mut hex = String::with_capacity(4);
                                for _ in 0..4 {
                                    match self.advance() {
                                        Some(c) if c.is_ascii_hexdigit() => {
                                            hex.push(c);
                                        }
                                        _ => {
                                            return Token::Error(format!(
                                                "{start_line}:{start_col} invalid \\u escape (need 4 hex digits)"
                                            ));
                                        }
                                    }
                                }
                                let val = u32::from_str_radix(&hex, 16).unwrap();
                                if (0xD800..=0xDFFF).contains(&val) {
                                    return Token::Error(format!(
                                        "{start_line}:{start_col} surrogate unicode escape"
                                    ));
                                }
                                s.push(char::from_u32(val).unwrap());
                            }
                        }
                        Some(c) => {
                            return Token::Error(format!(
                                "{start_line}:{start_col} invalid escape sequence: \\{c}"
                            ));
                        }
                        None => {
                            return Token::Error(format!(
                                "{start_line}:{start_col} unterminated string"
                            ));
                        }
                    }
                }
                _ => {
                    s.push(c);
                    self.advance();
                }
            }
        }
        Token::Error(format!("{start_line}:{start_col} unterminated string"))
    }

    fn lex_multiline_string(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;

        while let Some(c) = self.peek() {
            if c == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }

        let saved = self.pos;
        let mut depth = 0;
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else if c == '|' {
                depth += 1;
                self.advance();
            } else {
                break;
            }
        }
        self.pos = saved;

        let mut lines = Vec::new();

        loop {
            if self.is_at_end() {
                return Token::Error(format!(
                    "{start_line}:{start_col} unterminated multiline string"
                ));
            }

            let saved = self.pos;

            let mut pipes = 0;
            while let Some(c) = self.peek() {
                if c == ' ' || c == '\t' {
                    self.advance();
                } else if c == '|' && pipes < depth {
                    pipes += 1;
                    self.advance();
                } else {
                    break;
                }
            }
            if self.peek() == Some(' ') {
                self.advance();
            }

            if pipes == depth
                && self.peek() == Some('"')
                && self.input.get(self.pos + 1).copied() == Some('"')
                && self.input.get(self.pos + 2).copied() == Some('"')
            {
                self.advance();
                self.advance();
                self.advance();
                break;
            }

            self.pos = saved;
            let mut line = String::new();
            while let Some(c) = self.peek() {
                if c == '\n' {
                    self.advance();
                    break;
                }
                line.push(c);
                self.advance();
            }

            let stripped = strip_leading_pipes(&line, depth);
            lines.push(stripped);
        }

        let joined = lines.join("\n");
        match process_str_escapes(&joined) {
            Ok(processed) => Token::Str(processed),
            Err(msg) => Token::Error(format!("{start_line}:{start_col} {msg}")),
        }
    }

    fn lex_number(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if self.after_value_start {
            while let Some(c) = self.peek() {
                if c == '\n' || c == '#' {
                    break;
                }
                s.push(c);
                self.advance();
            }
            self.after_value_start = false;
            let trimmed = s.trim_end().to_string();
            return Self::typed_bare_value(&trimmed);
        }

        if is_spine_number(&s) {
            Token::Number(s.parse().unwrap_or(0.0))
        } else {
            Token::Str(s)
        }
    }

    fn lex_ident_or_keyword(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('"') {
            self.advance();
            return if let Some(content) = self.read_tagged_content() { Token::Tagged(s, content) } else {
                let msg = format!(
                    "{}:{} invalid escape in tagged literal",
                    self.line, self.col
                );
                Token::Error(msg)
            };
        }

        if self.peek() == Some('.') {
            let mut scan = self.pos;
            let mut found_tag_literal = false;
            while let Some(&sc) = self.input.get(scan) {
                if sc == '"' {
                    found_tag_literal = true;
                    break;
                }
                if sc == '\n' || sc == '#' {
                    break;
                }
                scan += 1;
            }
            if found_tag_literal {
                let tag_segment: String = self.input[self.pos..scan].iter().copied().collect();
                let is_valid = tag_segment.starts_with('.')
                    && tag_segment[1..].split('.').all(|part| {
                        !part.is_empty()
                            && part.chars().next().is_some_and(char::is_alphabetic)
                            && part
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                    });
                if is_valid {
                    let tag_suffix = &tag_segment[1..];
                    let full_tag = format!("{s}.{tag_suffix}");
                    while self.pos < scan {
                        self.advance();
                    }
                    self.advance();
                    return if let Some(content) = self.read_tagged_content() { Token::Tagged(full_tag, content) } else {
                        let msg = format!(
                            "{}:{} invalid escape in tagged literal",
                            self.line, self.col
                        );
                        Token::Error(msg)
                    };
                }
            }
        }

        if self.after_value_start {
            while let Some(c) = self.peek() {
                if c == '\n' || c == '#' {
                    break;
                }
                s.push(c);
                self.advance();
            }
            self.after_value_start = false;
            let trimmed = s.trim_end().to_string();
            return Self::typed_bare_value(&trimmed);
        }

        match s.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "null" => Token::Null,
            _ => Token::Ident(s),
        }
    }

    fn read_tagged_content(&mut self) -> Option<String> {
        let mut content = String::new();
        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.advance();
                    return Some(content);
                }
                '\\' => {
                    self.advance();
                    match self.advance() {
                        Some('n') => content.push('\n'),
                        Some('t') => content.push('\t'),
                        Some('r') => content.push('\r'),
                        Some('0') => content.push('\0'),
                        Some('\\') => content.push('\\'),
                        Some('"') => content.push('"'),
                        _ => return None,
                    }
                }
                _ => {
                    content.push(c);
                    self.advance();
                }
            }
        }
        None
    }

    fn typed_bare_value(s: &str) -> Token {
        if is_spine_number(s) {
            if let Ok(n) = s.parse::<f64>() {
                return Token::Number(n);
            }
        }
        match s {
            "true" => return Token::Bool(true),
            "false" => return Token::Bool(false),
            "null" => return Token::Null,
            _ => {}
        }
        Token::Str(s.to_string())
    }

    fn consume_bare_value(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '\n' || c == '#' {
                break;
            }
            s.push(c);
            self.advance();
        }
        self.after_value_start = false;
        let trimmed = s.trim_end().to_string();
        Self::typed_bare_value(&trimmed)
    }

    /// Tokenizes the entire input and returns a vector of spanned tokens.
    ///
    /// Tokens are produced in source order. The returned vector includes
    /// all significant tokens as well as comments and error tokens.
    #[allow(clippy::missing_panics_doc)]
    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let line = self.line;
            let col = self.col;
            match self.peek().unwrap() {
                ' ' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.after_value_start = false;
                    tokens.push((Token::Newline, line, col));
                    self.advance();
                    self.line_pipes = 0;
                    let mut lookahead = self.pos;
                    while let Some(&c) = self.input.get(lookahead) {
                        match c {
                            '|' => {
                                self.line_pipes += 1;
                                lookahead += 1;
                            }
                            ' ' | '\t' => {
                                lookahead += 1;
                            }
                            _ => break,
                        }
                    }
                }
                '|' => {
                    self.line_pipes += 1;
                    tokens.push((Token::Pipe, line, col));
                    self.advance();
                }
                '=' => {
                    self.after_value_start = true;
                    tokens.push((Token::Equals, line, col));
                    self.advance();
                }
                '~' => {
                    tokens.push((Token::Tilde, line, col));
                    self.advance();
                }
                '-' => {
                    if self.after_value_start {
                        let mut s = String::new();
                        s.push('-');
                        self.advance();
                        while let Some(c) = self.peek() {
                            if c == '\n' || c == '#' {
                                break;
                            }
                            s.push(c);
                            self.advance();
                        }
                        self.after_value_start = false;
                        let trimmed = s.trim_end().to_string();
                        tokens.push((Self::typed_bare_value(&trimmed), line, col));
                    } else {
                        tokens.push((Token::Dash, line, col));
                        self.advance();
                        self.after_value_start = true;
                    }
                }
                '.' => {
                    if self.after_value_start {
                        tokens.push((self.consume_bare_value(), line, col));
                    } else {
                        tokens.push((Token::Dot, line, col));
                        self.advance();
                    }
                }
                '#' => tokens.push((self.skip_line_comment(), line, col)),
                '/' => {
                    if self.input.get(self.pos + 1).copied() == Some('*') {
                        tokens.push((self.skip_block_comment(), line, col));
                    } else if self.after_value_start {
                        tokens.push((self.consume_bare_value(), line, col));
                    } else {
                        tokens.push((Token::Unknown('/'), line, col));
                        self.advance();
                    }
                }
                '"' => tokens.push((self.lex_string(), line, col)),
                c if c.is_ascii_digit() => tokens.push((self.lex_number(), line, col)),
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push((self.lex_ident_or_keyword(), line, col));
                }
                _ => {
                    if self.after_value_start {
                        tokens.push((self.consume_bare_value(), line, col));
                    } else {
                        let c = self.peek().unwrap();
                        self.advance();
                        tokens.push((Token::Unknown(c), line, col));
                    }
                }
            }
        }

        tokens
    }
}

fn is_spine_number(s: &str) -> bool {
    // Number grammar: ['-'] digit { digit } [ '.' digit { digit } ]
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    if bytes[i] == b'-' {
        i += 1;
    }
    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
        return false;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return false;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    i == bytes.len()
}

fn process_str_escapes(s: &str) -> Result<String, String> {
    let mut result = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match it.next() {
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some('0') => result.push('\0'),
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some('x') => {
                let (h1, h2) = (it.next(), it.next());
                match (h1, h2) {
                    (Some(h1), Some(h2)) if h1.is_ascii_hexdigit() && h2.is_ascii_hexdigit() => {
                        let val = (h1.to_digit(16).unwrap() << 4) | h2.to_digit(16).unwrap();
                        result.push(char::from_u32(val).unwrap());
                    }
                    _ => return Err("invalid \\x escape".into()),
                }
            }
            Some('u') => {
                if it.as_str().starts_with('{') {
                    it.next();
                    let mut hex = String::new();
                    loop {
                        match it.next() {
                            Some('}') => break,
                            Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                            _ => return Err("invalid \\u{} escape".into()),
                        }
                    }
                    if hex.is_empty() {
                        return Err("empty \\u{} escape".into());
                    }
                    let val = u32::from_str_radix(&hex, 16).unwrap();
                    if val > 0x0010_FFFF {
                        return Err("unicode escape out of range".into());
                    }
                    if (0xD800..=0xDFFF).contains(&val) {
                        return Err("surrogate unicode escape".into());
                    }
                    result.push(char::from_u32(val).unwrap());
                } else {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        match it.next() {
                            Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                            _ => return Err("invalid \\u escape (need 4 hex digits)".into()),
                        }
                    }
                    let val = u32::from_str_radix(&hex, 16).unwrap();
                    if (0xD800..=0xDFFF).contains(&val) {
                        return Err("surrogate unicode escape".into());
                    }
                    result.push(char::from_u32(val).unwrap());
                }
            }
            Some(c) => return Err(format!("invalid escape sequence: \\{c}")),
            None => return Err("unterminated escape at end of string".into()),
        }
    }
    Ok(result)
}

fn strip_leading_pipes(line: &str, depth: usize) -> String {
    let mut chars = line.chars().peekable();
    let mut stripped = 0;
    while stripped < depth {
        match chars.peek() {
            Some(' ' | '\t') => {
                chars.next();
            }
            Some('|') => {
                chars.next();
                stripped += 1;
            }
            _ => break,
        }
    }
    if chars.peek() == Some(&' ') {
        chars.next();
    }
    chars.collect()
}
