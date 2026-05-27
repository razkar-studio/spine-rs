use crate::{SpannedToken, Token, Value};

use farben::prelude::*;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    pub errors: Vec<String>,
    source: Option<String>,
    source_lines: Vec<String>,
    // (line, col, source_line)
    key_spans: std::collections::HashMap<String, (usize, usize, String)>,
}

impl Parser {
    #[must_use] 
    pub fn new(tokens: Vec<SpannedToken>, source_text: &str) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            source: None,
            source_lines: source_text.lines().map(std::string::ToString::to_string).collect(),
            key_spans: std::collections::HashMap::new(),
        }
    }

    #[must_use] 
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    fn get_source_line(&self, line: usize) -> &str {
        self.source_lines
            .get(line.saturating_sub(1))
            .map_or("", std::string::String::as_str)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _, _)| t)
    }

    fn peek_span(&self) -> Option<(usize, usize)> {
        self.tokens.get(self.pos).map(|(_, l, c)| (*l, *c))
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos).map(|(t, _, _)| t);
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
                Some(Token::Newline | Token::LineComment(_) | Token::BlockComment(_)) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn current_depth(&mut self) -> usize {
        let mut depth = 0;
        let mut lookahead = self.pos;
        while let Some((Token::Pipe, _, _)) = self.tokens.get(lookahead) {
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
                let (line, col) = self.peek_span().unwrap_or((0, 0));
                self.advance();
                self.parse_ident(name, fields, depth, line, col);
            }
            _ => {
                self.advance();
            }
        }
    }

    fn parse_ident(
        &mut self,
        name: String,
        fields: &mut Vec<(String, Value)>,
        depth: usize,
        line: usize,
        col: usize,
    ) {
        match self.peek().cloned() {
            Some(Token::Dot) => {
                self.advance();
                if let Some(Token::Ident(next)) = self.peek().cloned() {
                    self.advance();
                    let mut child_fields = Vec::new();
                    self.parse_ident(next, &mut child_fields, depth, line, col);
                    self.merge_into(fields, name, Value::Object(child_fields), line, col);
                }
            }
            Some(Token::Equals) => {
                self.advance();
                let value = self.parse_value();
                self.skip_newlines();
                self.merge_into(fields, name, value, line, col);
            }
            Some(Token::Newline) | None => {
                self.skip_newlines();

                let is_array = self.is_array_block(depth + 1);

                if is_array {
                    let mut entries = Vec::new();
                    while self.current_depth() == depth + 1 {
                        self.consume_pipes(depth + 1);
                        match self.peek().cloned() {
                            Some(Token::Dash) => {
                                self.advance();
                                self.skip_newlines();
                                if self.current_depth() == depth + 2 {
                                    let mut child_fields = Vec::new();
                                    while self.current_depth() == depth + 2 {
                                        self.parse_statement(&mut child_fields, depth + 2);
                                        self.skip_comments_and_newlines();
                                    }
                                    entries.push(Value::Object(child_fields));
                                } else {
                                    match self.peek().cloned() {
                                        Some(Token::Newline) | None => entries.push(Value::Null),
                                        _ => {
                                            let v = self.parse_value();
                                            entries.push(v);
                                        }
                                    }
                                }
                            }
                            _ => break,
                        }
                        self.skip_comments_and_newlines();
                    }
                    self.merge_into(fields, name, Value::Array(entries), line, col);
                } else {
                    let mut child_fields = Vec::new();
                    while self.current_depth() == depth + 1 {
                        self.parse_statement(&mut child_fields, depth + 1);
                        self.skip_comments_and_newlines();
                    }
                    self.merge_into(fields, name, Value::Object(child_fields), line, col);
                }
            }
            _ => {}
        }
    }

    fn is_array_block(&self, depth: usize) -> bool {
        let mut i = self.pos;
        let mut pipes = 0;
        while let Some((Token::Pipe, _, _)) = self.tokens.get(i) {
            pipes += 1;
            i += 1;
        }
        if pipes != depth {
            return false;
        }
        matches!(self.tokens.get(i), Some((Token::Dash, _, _)))
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

    pub fn parse(&mut self) -> Result<Value, Vec<String>> {
        let mut fields: Vec<(String, Value)> = Vec::new();

        self.skip_comments_and_newlines();

        while self.peek().is_some() {
            self.parse_statement(&mut fields, 0);
            self.skip_comments_and_newlines();
        }

        if self.errors.is_empty() {
            Ok(Value::Object(fields))
        } else {
            Err(self.errors.clone())
        }
    }

    // --- //

    fn format_error(
        &self,
        kind: &str,
        message: &str,
        // (line, col, source_line, token_len, note)
        lines: &[(usize, usize, &str, usize, Option<&str>)],
    ) -> String {
        let filename = self.source.as_deref().unwrap_or("<input>");

        let max_line_width = lines
            .iter()
            .map(|(l, _, _, _, _)| l.to_string().len())
            .max()
            .unwrap_or(1);

        let mut out = String::new();

        out += &color_fmt!("[dim]┌─[/] [bold red]error[/]: [bold]{}\n", kind);
        out += &color_fmt!("[dim]│[/]  {}\n", filename);

        for (line, col, source_line, token_len, note) in lines {
            let line_str = format!("{line:>max_line_width$}");
            let col_str = format!("{col}");
            let gutter_len = 3 + line_str.len() + 1 + col_str.len() + 1;
            let caret_pad = gutter_len + col;
            let carets = format!("{:>pad$}", "^".repeat(*token_len), pad = caret_pad);

            out += &color_fmt!(
                "[dim]├─[/] [cyan]{}:{}[/] {}\n",
                line_str,
                col_str,
                source_line
            );

            if let Some(note_text) = note {
                out += &color_fmt!("[dim]│[/]  [red]{}[/] [red]{}\n[/]", carets, note_text);
            } else {
                out += &color_fmt!("[dim]│[/]  [red]{}\n[/]", carets);
            }
        }

        out += &color_fmt!("[dim]└─[/] [bold]{}", message);

        out
    }

    fn merge_into(
        &mut self,
        fields: &mut Vec<(String, Value)>,
        key: String,
        value: Value,
        line: usize,
        col: usize,
    ) {
        if let Some(existing) = fields.iter_mut().find(|(k, _)| k == &key) {
            match (std::mem::take(&mut existing.1), value) {
                (Value::Object(mut a), Value::Object(b)) => {
                    for (k, v) in b {
                        self.merge_into(&mut a, k, v, line, col);
                    }
                    existing.1 = Value::Object(a);
                }
                (old, _new) => {
                    existing.1 = old;
                    let current_source = self.get_source_line(line).to_string();
                    let token_len = key.len();

                    let error = if let Some((first_line, first_col, first_source)) =
                        self.key_spans.get(&key).cloned()
                    {
                        self.format_error(
                            "duplicate-key",
                            &format!("'{key}' was already defined"),
                            &[
                                (
                                    first_line,
                                    first_col,
                                    first_source.as_str(),
                                    token_len,
                                    Some("first defined here"),
                                ),
                                (
                                    line,
                                    col,
                                    &current_source,
                                    token_len,
                                    Some("redefined here"),
                                ),
                            ],
                        )
                    } else {
                        self.format_error(
                            "duplicate-key",
                            &format!("'{key}' was already defined"),
                            &[(
                                line,
                                col,
                                &current_source,
                                token_len,
                                Some("redefined here"),
                            )],
                        )
                    };
                    self.errors.push(error);
                }
            }
        } else {
            let source_line = self.get_source_line(line).to_string();
            self.key_spans.insert(key.clone(), (line, col, source_line));
            fields.push((key, value));
        }
    }
}
