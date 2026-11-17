use crate::{HybridError, KEYWORDS, Spanned, Token};

/// The tokens of a program and the place the parser has reached.
pub struct Cursor {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Cursor {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    pub fn line(&self) -> usize {
        self.tokens[self.pos].line
    }

    pub fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        self.pos = (self.pos + 1).min(self.tokens.len() - 1);
        token
    }

    pub fn eat(&mut self, token: &Token) -> bool {
        let found = self.peek() == token;
        if found {
            self.bump();
        }
        found
    }

    pub fn at_word(&self, word: &str) -> bool {
        matches!(self.peek(), Token::Ident(w) if w == word)
    }

    pub fn eat_word(&mut self, word: &str) -> bool {
        let found = self.at_word(word);
        if found {
            self.bump();
        }
        found
    }

    pub fn expect(&mut self, token: &Token) -> Result<(), HybridError> {
        match self.eat(token) {
            true => Ok(()),
            false => Err(self.error(&token.to_string())),
        }
    }

    pub fn expect_word(&mut self, word: &str) -> Result<(), HybridError> {
        match self.eat_word(word) {
            true => Ok(()),
            false => Err(self.error(&format!("`{word}`"))),
        }
    }

    pub fn name(&mut self) -> Result<String, HybridError> {
        match self.peek().clone() {
            Token::Ident(name) if !KEYWORDS.contains(&name.as_str()) => {
                self.bump();
                Ok(name)
            }
            _ => Err(self.error("a name")),
        }
    }

    pub fn error(&self, expected: &str) -> HybridError {
        HybridError::Parse {
            line: self.line(),
            expected: expected.into(),
            found: self.peek().to_string(),
        }
    }
}
