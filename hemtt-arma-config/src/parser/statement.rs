use super::Node;

#[derive(Debug, Clone)]
pub enum Statement {
    Config(Vec<Node>),
    Array(Vec<Node>),
    Float(f32),
    Integer(i64),
    Str(String),
    Bool(bool),
    Property {
        ident: Box<Node>,
        value: Box<Node>,
        expand: bool,
    },
    Class {
        ident: Box<Node>,
        extends: Option<Box<Node>>,
        props: Vec<Node>,
    },
    ClassDef(Box<Node>),
    ClassDelete(Box<Node>),
    Ident(String),
    IdentArray(String),

    Gone,
}
