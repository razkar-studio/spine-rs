use crate::Token;

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.get(self.pos).copied();
        self.pos += 1;
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
        // placeholder
        let mut s = String::new();
        loop {
            match (
                self.peek(),
                self.input.get(self.pos + 1).copied(),
                self.input.get(self.pos + 2).copied(),
            ) {
                (Some('"'), Some('"'), Some('"')) => {
                    self.advance();
                    self.advance();
                    self.advance();
                    break;
                }
                (Some(c), _, _) => {
                    s.push(c);
                    self.advance();
                }
                _ => break,
            }
        }
        Token::Str(s.trim().to_string())
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

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            match self.peek().unwrap() {
                ' ' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    tokens.push(Token::Newline);
                    self.advance();
                }
                '|' => {
                    tokens.push(Token::Pipe);
                    self.advance();
                }
                '=' => {
                    tokens.push(Token::Equals);
                    self.advance();
                }
                '~' => {
                    tokens.push(Token::Tilde);
                    self.advance();
                }
                '-' => {
                    tokens.push(Token::Dash);
                    self.advance();
                }
                '.' => {
                    tokens.push(Token::Dot);
                    self.advance();
                }
                '#' => tokens.push(self.skip_line_comment()),
                '/' => tokens.push(self.skip_block_comment()),
                '"' => tokens.push(self.lex_string()),
                c if c.is_ascii_digit() => tokens.push(self.lex_number()),
                c if c.is_alphabetic() || c == '_' => tokens.push(self.lex_ident_or_keyword()),
                _ => {
                    self.advance();
                }
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut lexer = Lexer::new("server\n| host = localhost\n| port = 8080\n");
        let tokens = lexer.tokenize();
        println!("{:?}", tokens);
    }
}
