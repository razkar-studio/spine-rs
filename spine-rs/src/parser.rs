use crate::{SpannedToken, Token, Value};

use farben::prelude::*;
use unicode_width::UnicodeWidthStr;

type Spans = std::collections::HashMap<String, (usize, usize, String)>;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    pub errors: Vec<String>,
    source: Option<String>,
    source_lines: Vec<String>,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<SpannedToken>, source_text: &str) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            source: None,
            source_lines: source_text
                .lines()
                .map(std::string::ToString::to_string)
                .collect(),
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
        self.skip_comments_and_newlines();
    }

    fn skip_comments_and_newlines(&mut self) {
        loop {
            match self.peek() {
                Some(Token::Newline | Token::LineComment(_) | Token::BlockComment(_)) => {
                    self.advance();
                }
                Some(Token::Pipe) => {
                    let mut lookahead = self.pos;
                    while let Some((Token::Pipe, _, _)) = self.tokens.get(lookahead) {
                        lookahead += 1;
                    }

                    match self.tokens.get(lookahead) {
                        Some((
                            Token::Newline | Token::LineComment(_) | Token::BlockComment(_),
                            _,
                            _,
                        )) => {
                            while self.pos <= lookahead {
                                self.advance();
                            }
                        }
                        None => {
                            while self.pos < lookahead {
                                self.advance();
                            }
                            break;
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }
    }

    fn current_depth(&self) -> usize {
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

    fn parse_statement(
        &mut self,
        fields: &mut Vec<(String, Value)>,
        spans: &mut Spans,
        depth: usize,
    ) {
        if self.current_depth() < depth {
            return;
        }
        self.consume_pipes(depth);

        match self.peek().cloned() {
            Some(Token::Tilde) => {
                let (line, col) = self.peek_span().unwrap_or((0, 0));
                self.advance();
                self.parse_append(fields, spans, depth, line, col);
            }
            Some(Token::Ident(name)) => {
                let (line, col) = self.peek_span().unwrap_or((0, 0));
                self.advance();
                self.parse_ident(name, fields, spans, depth, line, col);
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
        spans: &mut Spans,
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
                    let mut child_spans = Spans::new();
                    self.parse_ident(next, &mut child_fields, &mut child_spans, depth, line, col);
                    self.merge_into(
                        fields,
                        spans,
                        name,
                        Value::Object(child_fields),
                        line,
                        col,
                        depth,
                    );
                } else {
                    let current_source = self.get_source_line(line).to_string();
                    let error = self.format_error(
                        "syntax-error",
                        &format!("expected identifier after '.' in '{name}.'"),
                        &[(
                            line,
                            col,
                            &current_source,
                            name.len() + 1,
                            Some("incomplete dotted path"),
                        )],
                    );
                    self.errors.push(error);
                }
            }
            Some(Token::Equals) => {
                self.advance();
                let value = self.parse_value();
                self.skip_newlines();
                self.merge_into(fields, spans, name, value, line, col, depth);
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
                                    let mut child_spans = Spans::new();
                                    while self.current_depth() == depth + 2 {
                                        self.parse_statement(
                                            &mut child_fields,
                                            &mut child_spans,
                                            depth + 2,
                                        );
                                        self.skip_comments_and_newlines();
                                    }
                                    entries.push(Value::Object(child_fields));
                                } else {
                                    match self.peek().cloned() {
                                        Some(Token::Newline) | None => entries.push(Value::Null),
                                        Some(Token::Ident(name) | Token::Str(name)) => {
                                            let (line, col) = self.peek_span().unwrap_or((0, 0));
                                            self.advance();

                                            let has_children = match self.peek() {
                                                Some(Token::Equals | Token::Dot) => true,
                                                Some(Token::Newline) | None => {
                                                    let mut lookahead = self.pos;
                                                    while let Some((
                                                        Token::Newline
                                                        | Token::LineComment(_)
                                                        | Token::BlockComment(_),
                                                        _,
                                                        _,
                                                    )) = self.tokens.get(lookahead)
                                                    {
                                                        lookahead += 1;
                                                    }
                                                    let mut next_depth = 0;
                                                    while let Some((Token::Pipe, _, _)) =
                                                        self.tokens.get(lookahead)
                                                    {
                                                        next_depth += 1;
                                                        lookahead += 1;
                                                    }
                                                    next_depth == depth + 2
                                                }
                                                _ => false,
                                            };

                                            if has_children {
                                                let mut temp_fields = Vec::new();
                                                let mut temp_spans = Spans::new();
                                                self.parse_ident(
                                                    name,
                                                    &mut temp_fields,
                                                    &mut temp_spans,
                                                    depth + 1,
                                                    line,
                                                    col,
                                                );
                                                entries.push(Value::Object(temp_fields));
                                            } else {
                                                entries.push(Value::String(name));
                                            }
                                        }
                                        _ => {
                                            let v = self.parse_value();
                                            entries.push(v);
                                        }
                                    }
                                }
                            }
                            Some(Token::Newline) => {
                                self.advance();
                                self.skip_comments_and_newlines();
                                continue;
                            }
                            _ => break,
                        }
                        self.skip_comments_and_newlines();
                    }
                    self.merge_into(fields, spans, name, Value::Array(entries), line, col, depth);
                } else {
                    let mut child_fields = Vec::new();
                    let mut child_spans = Spans::new();
                    while self.current_depth() == depth + 1 {
                        self.parse_statement(&mut child_fields, &mut child_spans, depth + 1);
                        self.skip_comments_and_newlines();
                    }
                    self.merge_into(
                        fields,
                        spans,
                        name,
                        Value::Object(child_fields),
                        line,
                        col,
                        depth,
                    );
                }
            }
            _ => {
                let current_source = self.get_source_line(line).to_string();
                let error = self.format_error(
                    "syntax-error",
                    &format!("unexpected token after '{name}'"),
                    &[(
                        line,
                        col,
                        &current_source,
                        name.len(),
                        Some("expected '=', '.', or newline"),
                    )],
                );
                self.errors.push(error);
            }
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
            Some(Token::Str(s) | Token::Ident(s)) => {
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
            _ => Value::Null,
        }
    }

    fn parse_append(
        &mut self,
        fields: &mut Vec<(String, Value)>,
        spans: &mut Spans,
        depth: usize,
        tilde_line: usize,
        tilde_col: usize,
    ) {
        let mut path = Vec::new();
        if let Some(Token::Ident(name)) = self.peek().cloned() {
            self.advance();
            path.push(name);
            while self.peek() == Some(&Token::Dot) {
                self.advance();
                if let Some(Token::Ident(next)) = self.peek().cloned() {
                    self.advance();
                    path.push(next);
                } else {
                    break;
                }
            }
        } else {
            let source = self.get_source_line(tilde_line).to_string();
            let error = self.format_error(
                "syntax-error",
                "append requires a path after '~'",
                &[(
                    tilde_line,
                    tilde_col,
                    &source,
                    1,
                    Some("'~' used here"),
                )],
            );
            self.errors.push(error);
            return;
        }

        self.skip_newlines();

        let mut child_fields = Vec::new();
        let mut child_spans = Spans::new();
        while self.current_depth() == depth + 1 {
            self.parse_statement(&mut child_fields, &mut child_spans, depth + 1);
            self.skip_comments_and_newlines();
        }

        if child_fields.is_empty() {
            let source = self.get_source_line(tilde_line).to_string();
            let tilde_len: usize =
                path.iter().map(std::string::String::len).sum::<usize>() + path.len();
            let error = self.format_error(
                "syntax-error",
                "append requires child statements after the path",
                &[(
                    tilde_line,
                    tilde_col,
                    &source,
                    tilde_len,
                    Some("append path ends here"),
                )],
            );
            self.errors.push(error);
            return;
        }
        let entry = Value::Object(child_fields);

        let (prefix, last) = path.split_at(path.len() - 1);
        let last = &last[0];

        let mut temp_spans = Spans::new();
        let mut current_fields = fields;
        let mut current_spans: &mut Spans = spans;
        for segment in prefix {
            if !current_fields.iter().any(|(k, _)| k == segment) {
                current_fields.push((segment.clone(), Value::Object(Vec::new())));
            }
            let existing = current_fields
                .iter_mut()
                .find(|(k, _)| k == segment)
                .unwrap();
            if let Value::Object(ref mut inner) = existing.1 {
                current_fields = inner;
                current_spans = &mut temp_spans;
            } else {
                let current_source = self.get_source_line(tilde_line).to_string();
                let tilde_len: usize =
                    path.iter().map(std::string::String::len).sum::<usize>() + path.len();
                let error = self.format_error(
                    "type-conflict",
                    &format!("'{segment}' is not an object"),
                    &[(
                        tilde_line,
                        tilde_col,
                        &current_source,
                        tilde_len,
                        Some("append attempted here"),
                    )],
                );
                self.errors.push(error);
                return;
            }
        }

        if let Some(existing) = current_fields.iter_mut().find(|(k, _)| k == last) {
            if let Value::Array(ref mut arr) = existing.1 {
                arr.push(entry);
            } else {
                let current_source = self.get_source_line(tilde_line).to_string();
                let tilde_len: usize =
                    path.iter().map(std::string::String::len).sum::<usize>() + path.len();
                let error = if let Some((first_line, first_col, first_source)) =
                    current_spans.get(last).cloned()
                {
                    self.format_error(
                        "type-conflict",
                        &format!("'{last}' is not an array"),
                        &[
                            (
                                first_line,
                                first_col,
                                first_source.as_str(),
                                last.len(),
                                Some("first defined here as scalar"),
                            ),
                            (
                                tilde_line,
                                tilde_col,
                                &current_source,
                                tilde_len,
                                Some("append attempted here"),
                            ),
                        ],
                    )
                } else {
                    self.format_error(
                        "type-conflict",
                        &format!("'{last}' is not an array"),
                        &[(
                            tilde_line,
                            tilde_col,
                            &current_source,
                            tilde_len,
                            Some("append attempted here"),
                        )],
                    )
                };
                self.errors.push(error);
            }
        } else {
            current_fields.push((last.clone(), Value::Array(vec![entry])));
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn parse(&mut self) -> Result<Value, Vec<String>> {
        let mut fields: Vec<(String, Value)> = Vec::new();
        let mut spans = Spans::new();

        if let Some((Token::Unknown(c), line, col)) = self
            .tokens
            .iter()
            .find(|(t, _, _)| matches!(t, Token::Unknown(_)))
        {
            let source = self.get_source_line(*line).to_string();
            let error = self.format_error(
                "unexpected-character",
                &format!("unexpected character '{c}', is this a Spine file?"),
                &[(*line, *col, &source, 1, None)],
            );
            self.errors.push(error);
        }

        if let Some((Token::Error(msg), line, col)) = self
            .tokens
            .iter()
            .find(|(t, _, _)| matches!(t, Token::Error(_)))
        {
            let source = self.get_source_line(*line).to_string();
            let error = self.format_error("lexer-error", msg, &[(*line, *col, &source, 1, None)]);
            self.errors.push(error);
        }

        self.skip_comments_and_newlines();

        let base_depth = self.current_depth();

        while self.peek().is_some() {
            self.parse_statement(&mut fields, &mut spans, base_depth);
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
        lines: &[(usize, usize, &str, usize, Option<&str>)],
    ) -> String {
        let filename = self.source.as_deref().unwrap_or("<input>");

        let mut out = String::new();

        out += &color_fmt!(
            "[dim]┌─[/] [bold red]error[/red]: {}\n",
            farben_escape(kind.to_string())
        );
        out += &color_fmt!(
            "[dim]│[/]  [cyan]-->[/] {}\n",
            farben_escape(filename.to_string())
        );

        for (line, col, source_line, token_len, note) in lines {
            let gutter = format!("{line}:{col}");

            out += &color_fmt!(
                "[dim]├─[/] [cyan]{}[/] {}\n",
                gutter,
                farben_escape(source_line.to_string())
            );

            let start = col.saturating_sub(1);
            let mut caret_line = String::new();
            let gutter_width = UnicodeWidthStr::width(gutter.as_str());
            caret_line.push_str(&" ".repeat(gutter_width + 1));
            let prefix_chars = source_line.chars().take(start).count();
            caret_line.push_str(&" ".repeat(prefix_chars));
            caret_line.push_str(&"^".repeat(*token_len.max(&1)));

            if let Some(note_text) = note {
                out += &color_fmt!(
                    "[dim]│[/]  [red]{} {}[/]\n",
                    caret_line,
                    farben_escape(note_text.to_string())
                );
            } else {
                out += &color_fmt!("[dim]│[/]  [red]{}[/]\n", caret_line);
            }
        }

        out += &color_fmt!("[dim]└─[/] [bold]{}", farben_escape(message.to_string()));

        out
    }

    fn merge_into(
        &mut self,
        fields: &mut Vec<(String, Value)>,
        spans: &mut Spans,
        key: String,
        value: Value,
        line: usize,
        col: usize,
        debug_depth: usize,
    ) {
        if let Some(existing) = fields.iter_mut().find(|(k, _)| k == &key) {
            match (std::mem::take(&mut existing.1), value) {
                (Value::Object(mut a), Value::Object(b)) => {
                    let mut child_spans = Spans::new();
                    for (k, v) in b {
                        self.merge_into(&mut a, &mut child_spans, k, v, line, col, debug_depth + 1);
                    }
                    existing.1 = Value::Object(a);
                }
                (old, new) => {
                    existing.1 = old;
                    let current_source = self.get_source_line(line).to_string();
                    let token_len = key.len();

                    let is_type_conflict = matches!(
                        (&existing.1, &new),
                        (Value::Object(_), _) | (_, Value::Object(_))
                    );

                    let (kind, message) = if is_type_conflict {
                        (
                            "type-conflict",
                            format!("'{key}' cannot be both a scalar and an object"),
                        )
                    } else {
                        ("duplicate-key", format!("'{key}' was already defined"))
                    };

                    let error = if let Some((first_line, first_col, first_source)) =
                        spans.get(&key).cloned()
                    {
                        self.format_error(
                            kind,
                            &message,
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
                            kind,
                            &message,
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
            spans.insert(key.clone(), (line, col, source_line));
            fields.push((key, value));
        }
    }
}

fn farben_escape(input: String) -> String {
    input.replace("[", "\\[")
}
