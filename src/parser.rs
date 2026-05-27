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
}
