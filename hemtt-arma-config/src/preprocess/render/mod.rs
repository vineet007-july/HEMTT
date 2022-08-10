use std::collections::HashMap;

pub use self::rendered::{LineMap, Rendered};

use super::{token::Token, TokenPos};

mod html;
mod rendered;

pub fn render(source: Vec<TokenPos>) -> Rendered {
    let mut map = HashMap::new();
    let mut line = Vec::new();
    let mut lc = 1;
    let mut cc = 1;
    for token in &source {
        if token.token() == &Token::Newline {
            map.insert(lc, line);
            lc += 1;
            cc = 1;
            line = Vec::new();
        } else {
            line.push((
                cc,
                token.to_string().len(),
                token.path().to_owned(),
                token.start().1,
                token.end().1,
                token.token().clone(),
            ));
            cc += token.to_string().len();
        }
    }
    Rendered::new(source, map)
}
