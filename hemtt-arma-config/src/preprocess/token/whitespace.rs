use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Whitespace {
    Space,
    Tab,
}

impl ToString for Whitespace {
    fn to_string(&self) -> String {
        match self {
            Whitespace::Space => " ",
            Whitespace::Tab => "\t",
        }
        .to_string()
    }
}
