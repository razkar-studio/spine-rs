use crate::{Token, Value};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    fn skip_newlines(&mut self) {
        while self.peek() == Some(&Token::Newline) {
            self.advance();
        }
    }

    fn skip_comments_and_newlines(&mut self) {
        loop {
            match self.peek() {
                Some(Token::Newline)
                | Some(Token::LineComment(_))
                | Some(Token::BlockComment(_)) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn current_depth(&mut self) -> usize {
        let mut depth = 0;
        let mut lookahead = self.pos;
        while let Some(Token::Pipe) = self.tokens.get(lookahead) {
            depth += 1;
            lookahead += 1;
        }
        depth
    }

    fn consume_pipes(&mut self, depth: usize) {
        for _ in 0..depth {
            self.advance();
        }
    }

    fn parse_statement(&mut self, fields: &mut Vec<(String, Value)>, depth: usize) {
        if self.current_depth() < depth {
            return;
        }
        self.consume_pipes(depth);

        match self.peek().cloned() {
            Some(Token::Tilde) => {
                self.advance();
                self.parse_append(fields, depth);
            }
            Some(Token::Dash) => {
                self.advance();
            }
            Some(Token::Ident(name)) => {
                self.advance();
                self.parse_ident(name, fields, depth);
            }
            _ => {
                self.advance();
            }
        }
    }

    fn parse_ident(&mut self, name: String, fields: &mut Vec<(String, Value)>, depth: usize) {
        match self.peek().cloned() {
            Some(Token::Dot) => {
                self.advance();
                if let Some(Token::Ident(next)) = self.peek().cloned() {
                    self.advance();
                    let mut child_fields = Vec::new();
                    self.parse_ident(next, &mut child_fields, depth);
                    merge_into(fields, name, Value::Object(child_fields));
                }
            }
            Some(Token::Equals) => {
                self.advance();
                let value = self.parse_value();
                self.skip_newlines();
                merge_into(fields, name, value);
            }
            Some(Token::Newline) | None => {
                self.skip_newlines();
                let mut child_fields = Vec::new();
                while self.current_depth() == depth + 1 {
                    self.parse_statement(&mut child_fields, depth + 1);
                    self.skip_comments_and_newlines();
                }
                merge_into(fields, name, Value::Object(child_fields));
            }
            _ => {}
        }
    }

    fn parse_value(&mut self) -> Value {
        match self.peek().cloned() {
            Some(Token::Str(s)) => {
                self.advance();
                Value::String(s)
            }
            Some(Token::Number(n)) => {
                self.advance();
                Value::Number(n)
            }
            Some(Token::Bool(b)) => {
                self.advance();
                Value::Bool(b)
            }
            Some(Token::Null) => {
                self.advance();
                Value::Null
            }
            Some(Token::Tagged(tag, content)) => {
                self.advance();
                Value::Tagged(tag, content)
            }
            Some(Token::Ident(s)) => {
                self.advance();
                Value::String(s)
            }
            _ => Value::Null,
        }
    }

    fn parse_append(&mut self, fields: &mut Vec<(String, Value)>, depth: usize) {
        if let Some(Token::Ident(name)) = self.peek().cloned() {
            self.advance();
            self.skip_newlines();

            let mut child_fields = Vec::new();
            while self.current_depth() == depth + 1 {
                self.parse_statement(&mut child_fields, depth + 1);
                self.skip_comments_and_newlines();
            }

            let entry = if child_fields.is_empty() {
                Value::Null
            } else {
                Value::Object(child_fields)
            };

            if let Some(existing) = fields.iter_mut().find(|(k, _)| k == &name) {
                if let Value::Array(ref mut arr) = existing.1 {
                    arr.push(entry);
                } else {
                    panic!("conflict: '{name}' is not an array");
                }
            } else {
                fields.push((name, Value::Array(vec![entry])));
            }
        }
    }

    pub fn parse(&mut self) -> Value {
        let mut fields: Vec<(String, Value)> = Vec::new();

        self.skip_comments_and_newlines();

        while self.peek().is_some() {
            self.parse_statement(&mut fields, 0);
            self.skip_comments_and_newlines();
        }

        Value::Object(fields)
    }
}

fn merge_into(fields: &mut Vec<(String, Value)>, key: String, value: Value) {
    if let Some(existing) = fields.iter_mut().find(|(k, _)| k == &key) {
        let merged = match (std::mem::take(&mut existing.1), value) {
            (Value::Object(mut a), Value::Object(b)) => {
                for (k, v) in b {
                    merge_into(&mut a, k, v);
                }
                Value::Object(a)
            }
            _ => panic!("conflict: duplicate key '{key}'"),
        };
        existing.1 = merged;
    } else {
        fields.push((key, value));
    }
}
