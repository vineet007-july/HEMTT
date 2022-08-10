use pest::iterators::Pair;

use crate::preprocess::LineCol;

use super::{Rule, Token};

#[derive(Clone, Debug)]
pub struct TokenPos {
    start: LineCol,
    end: LineCol,
    path: String,
    token: Token,
}

impl TokenPos {
    pub fn new<S: Into<String>>(
        path: S,
        pair: Pair<'_, Rule>,
        start: LineCol,
        end: LineCol,
    ) -> Self {
        Self {
            start,
            end,
            path: path.into(),
            token: Token::from(pair),
        }
    }

    pub fn anon(token: Token) -> Self {
        Self {
            start: (0, (0, 0)),
            end: (0, (0, 0)),
            path: String::new(),
            token,
        }
    }

    pub fn with_pos(token: Token, pos: &Self) -> Self {
        Self {
            start: pos.start(),
            end: pos.end(),
            path: pos.path().to_string(),
            token,
        }
    }

    pub fn start(&self) -> LineCol {
        self.start
    }

    pub fn end(&self) -> LineCol {
        self.end
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn into_token(self) -> Token {
        self.token
    }
}

impl ToString for TokenPos {
    fn to_string(&self) -> String {
        self.token().to_string()
    }
}
