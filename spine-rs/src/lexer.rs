use crate::{SpannedToken, Token};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line_pipes: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line_pipes: 0,
            line: 1,
            col: 1,
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
                        break;
                    }
                    content.push_str("*/");
                }
                (Some(c), _) => {
                    content.push(c);
                    self.advance();
                }
                _ => break,
            }
        }
        Token::BlockComment(content.trim().to_string())
    }

    fn lex_string(&mut self) -> Token {
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
                    break;
                }
                '\\' => {
                    self.advance();
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        _ => {}
                    }
                }
                _ => {
                    s.push(c);
                    self.advance();
                }
            }
        }
        Token::Str(s)
    }

    fn lex_multiline_string(&mut self) -> Token {
        let depth = self.line_pipes;

        while let Some(c) = self.peek() {
            if c == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }

        let mut lines = Vec::new();

        loop {
            if self.is_at_end() {
                break;
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

            if self.peek() == Some('"')
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

        Token::Str(lines.join("\n"))
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
        Token::Number(s.parse().unwrap_or(0.0))
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

        if self.peek() == Some('.') {
            if let Some(&next) = self.input.get(self.pos + 1) {
                if next.is_alphabetic() {
                    let mut lookahead = self.pos + 1;
                    while let Some(&c) = self.input.get(lookahead) {
                        if c.is_alphanumeric() || c == '_' || c == '-' {
                            lookahead += 1;
                        } else {
                            break;
                        }
                    }
                    if self.input.get(lookahead) == Some(&'"') {
                        self.advance();
                        let mut tag_suffix = String::new();
                        while let Some(c) = self.peek() {
                            if c.is_alphanumeric() || c == '_' || c == '-' {
                                tag_suffix.push(c);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        let full_tag = format!("{}.{}", s, tag_suffix);
                        self.advance();
                        let mut content = String::new();
                        while let Some(c) = self.peek() {
                            if c == '"' {
                                self.advance();
                                break;
                            }
                            content.push(c);
                            self.advance();
                        }
                        return Token::Tagged(full_tag, content);
                    }
                }
            }
        }

        if self.peek() == Some('"') {
            self.advance();
            let mut content = String::new();
            while let Some(c) = self.peek() {
                if c == '"' {
                    self.advance();
                    break;
                }
                content.push(c);
                self.advance();
            }
            return Token::Tagged(s, content);
        }

        match s.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "null" => Token::Null,
            _ => Token::Ident(s),
        }
    }

    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let line = self.line;
            let col = self.col;
            match self.peek().unwrap() {
                ' ' | '\t' => {
                    self.advance();
                    continue;
                }
                '\n' => {
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
                    tokens.push((Token::Equals, line, col));
                    self.advance();
                }
                '~' => {
                    tokens.push((Token::Tilde, line, col));
                    self.advance();
                }
                '-' => {
                    tokens.push((Token::Dash, line, col));
                    self.advance();
                }
                '.' => {
                    tokens.push((Token::Dot, line, col));
                    self.advance();
                }
                '#' => tokens.push((self.skip_line_comment(), line, col)),
                '/' => tokens.push((self.skip_block_comment(), line, col)),
                '"' => tokens.push((self.lex_string(), line, col)),
                c if c.is_ascii_digit() => tokens.push((self.lex_number(), line, col)),
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push((self.lex_ident_or_keyword(), line, col))
                }
                _ => {
                    self.advance();
                }
            }
        }

        tokens
    }
}

fn strip_leading_pipes(line: &str, depth: usize) -> String {
    let mut chars = line.chars().peekable();
    let mut stripped = 0;
    while stripped < depth {
        match chars.peek() {
            Some(' ') | Some('\t') => {
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
