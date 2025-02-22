use std::collections::HashMap;
use std::iter::Peekable;
use std::vec::IntoIter;

use pest::error::Error;
use pest::Parser;

mod token;
#[cfg(test)]
use token::Whitespace;
pub use token::{PreProcessParser, Rule, Token, TokenPos};

mod render;
pub use render::{render, LineMap};

mod define;
use define::Define;
mod ifstate;
use ifstate::{IfState, IfStates};

use crate::{resolver::Resolver, ArmaConfigError};

pub type LineCol = (usize, (usize, usize));

pub fn tokenize(source: &str, path: &str) -> Result<Vec<TokenPos>, Error<Rule>> {
    let mut tokens = Vec::new();

    let pairs = PreProcessParser::parse(Rule::file, source)?;
    let mut line = 1;
    let mut col = 1;
    let mut offset = 0;
    for pair in pairs {
        let start = (offset, (line, col));
        if let Rule::newline = pair.as_rule() {
            line += 1;
            col = 1;
        } else {
            col += pair.as_str().len();
        }
        offset += pair.as_str().len();
        let end = (offset, (line, col));
        tokens.push(TokenPos::new(path, pair, start, end));
    }

    Ok(tokens)
}

macro_rules! skip_whitespace {
    ($i: ident) => {{
        let mut next = $i.peek();
        loop {
            if let Some(tp) = next {
                if tp.token().is_whitespace() {
                    $i.next();
                    next = $i.peek();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }};
}

macro_rules! read_args {
    ($i: ident) => {{
        let mut ret: Vec<Vec<TokenPos>> = Vec::new();
        let mut next = $i.next();
        let mut arg: Vec<TokenPos> = Vec::new();
        let mut level = 0;
        if let Some(ref tp) = next {
            if let Token::LeftParenthesis = tp.token() {
                next = $i.next();
            }
        }
        loop {
            if let Some(tp) = next {
                match tp.token() {
                    Token::LeftParenthesis => {
                        level += 1;
                        arg.push(TokenPos::anon(Token::LeftParenthesis));
                    }
                    Token::RightParenthesis => {
                        if level == 0 {
                            if !arg.is_empty() {
                                ret.push(arg);
                            }
                            break;
                        } else {
                            arg.push(TokenPos::anon(Token::RightParenthesis));
                        }
                        level -= 1;
                    }
                    Token::Comma => {
                        if level == 0 {
                            if !arg.is_empty() {
                                ret.push(arg);
                                arg = Vec::new();
                            }
                        } else {
                            arg.push(TokenPos::anon(Token::Comma));
                        }
                    }
                    _ => arg.push(tp),
                }
            } else {
                break;
            }
            next = $i.next();
        }
        ret
    }};
}

macro_rules! read_line {
    ($i: ident) => {{
        let mut ret: Vec<TokenPos> = Vec::new();
        let mut next = $i.next();
        // Skip initial whitespace
        loop {
            if let Some(tp) = &next {
                if tp.token().is_whitespace() {
                    next = $i.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let mut is_quoted = false;
        loop {
            if let Some(tp) = next {
                if is_quoted {
                    if tp.token() == &Token::DoubleQuote {
                        is_quoted = false;
                    }
                    ret.push(tp);
                    next = $i.next();
                } else {
                    match tp.token() {
                        Token::Newline => break,
                        Token::Escape => {
                            let mut skip = false;
                            if let Some(tp) = $i.peek() {
                                if tp.token() == &Token::Newline {
                                    ret.push(tp.clone());
                                    $i.next();
                                    skip = true;
                                }
                            }
                            next = $i.next();
                            if !skip {
                                let mut first = true;
                                loop {
                                    if let Some(ref ntp) = next {
                                        if ntp.token().is_whitespace() {
                                            next = $i.next();
                                            first = false;
                                        } else if first {
                                            ret.push(tp.clone());
                                            break;
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                        Token::DoubleQuote => {
                            ret.push(tp);
                            next = $i.next();
                            is_quoted = true;
                        }
                        _ => {
                            ret.push(tp);
                            next = $i.next();
                        }
                    }
                }
            } else {
                break;
            }
        }
        ret
    }};
}

pub fn _resolve<R>(
    ident: &str,
    define: &Define,
    root: &str,
    resolver: R,
    defines: &HashMap<String, Define>,
) -> Result<Option<Vec<TokenPos>>, ArmaConfigError>
where
    R: Resolver,
{
    Ok(if let Some(d) = defines.get(ident) {
        let mut ret = Vec::new();
        let mut context = defines.to_owned();
        if let Some(dargs) = &d.args {
            if let Some(args) = &define.args {
                if dargs.len() != args.len() {
                    return Err(ArmaConfigError::ArgCoundMismatch {
                        expected: args.len(),
                        actual: dargs.len(),
                        args: args.iter().map(|a| render(a.to_vec()).export()).collect(),
                    });
                }
                for i in 0..dargs.len() {
                    if let Token::Word(key) = &dargs[i][0].token() {
                        if args[i].len() == 1 {
                            if let Token::Word(value) = &args[i][0].token() {
                                context.insert(
                                    key.to_owned(),
                                    if let Some(ed) = defines.get(value) {
                                        ed.to_owned()
                                    } else {
                                        Define {
                                            args: None,
                                            statement: vec![args[i][0].to_owned()],
                                            call: false,
                                        }
                                    },
                                );
                            }
                        } else {
                            context.insert(
                                key.to_owned(),
                                Define {
                                    args: None,
                                    statement: args[i].to_owned(),
                                    call: false,
                                },
                            );
                        }
                    }
                }
            }
        }
        let mut iter = d.statement.clone().into_iter().peekable();
        while let Some(token) = iter.next() {
            match &token.token() {
                Token::Directive => {
                    if let Some(tp) = iter.peek() {
                        match tp.token() {
                            Token::Word(_) => {
                                if let Token::Word(w) = iter.next().unwrap().token() {
                                    ret.push(TokenPos::with_pos(Token::DoubleQuote, &token));
                                    ret.append(&mut _resolve_word(
                                        &mut iter,
                                        w,
                                        &token,
                                        root,
                                        resolver.clone(),
                                        &mut context,
                                    )?);
                                    ret.push(TokenPos::with_pos(Token::DoubleQuote, &token));
                                }
                            }
                            Token::Directive => {
                                iter.next();
                            }
                            _ => {}
                        }
                    }
                }
                Token::Word(w) => {
                    ret.append(&mut _resolve_word(
                        &mut iter,
                        w,
                        &token,
                        root,
                        resolver.clone(),
                        &mut context,
                    )?);
                }
                _ => ret.push(token.to_owned()),
            }
        }
        Some(ret)
    } else {
        None
    })
}

fn _resolve_word<R>(
    iter: &mut Peekable<IntoIter<TokenPos>>,
    ident: &str,
    token: &TokenPos,
    root: &str,
    resolver: R,
    defines: &mut HashMap<String, Define>,
) -> Result<Vec<TokenPos>, ArmaConfigError>
where
    R: Resolver,
{
    if let Some(d2) = defines.get(ident) {
        if d2.call {
            if let Some(r) = _resolve(
                ident,
                &Define {
                    call: false,
                    args: Some(
                        read_args!(iter)
                            .into_iter()
                            .map(|arg| _preprocess(arg, root, resolver.clone(), defines))
                            .collect::<Result<Vec<Vec<TokenPos>>, ArmaConfigError>>()
                            .unwrap(),
                    ),
                    statement: Vec::new(),
                },
                root,
                resolver,
                defines,
            )? {
                return Ok(r);
            }
        } else if let Some(r) = _resolve(ident, d2, root, resolver, defines)? {
            return Ok(r);
        } else {
            return Ok(vec![token.to_owned()]);
        }
    }
    Ok(vec![token.to_owned()])
}

pub fn preprocess<R>(
    source: Vec<TokenPos>,
    root: &str,
    resolver: R,
) -> Result<Vec<TokenPos>, ArmaConfigError>
where
    R: Resolver,
{
    let mut defines: HashMap<String, Define> = HashMap::new();
    _preprocess(source, root, resolver, &mut defines)
}

pub fn _preprocess<R>(
    source: Vec<TokenPos>,
    root: &str,
    resolver: R,
    defines: &mut std::collections::HashMap<std::string::String, define::Define>,
) -> Result<Vec<TokenPos>, ArmaConfigError>
where
    R: Resolver,
{
    let mut ret = Vec::new();
    let mut iter = source.into_iter().peekable();
    let mut if_state = IfStates::new();
    let mut new_line = true;
    while let Some(token) = iter.next() {
        match (&token.token(), if_state.reading(), new_line) {
            (Token::Directive, r, true) => {
                if let Token::Word(directive) = iter.next().unwrap().token() {
                    match (directive.as_str(), r) {
                        ("define", true) => {
                            skip_whitespace!(iter);
                            if let Some(tp) = iter.next() {
                                if let Token::Word(name) = tp.token() {
                                    // skip_whitespace!(iter);
                                    let args = if let Some(tp) = iter.peek() {
                                        if tp.token() == &Token::LeftParenthesis {
                                            let args = read_args!(iter)
                                                .into_iter()
                                                .map(|arg| {
                                                    _preprocess(
                                                        arg,
                                                        root,
                                                        resolver.clone(),
                                                        defines,
                                                    )
                                                })
                                                .collect::<Result<Vec<Vec<TokenPos>>, ArmaConfigError>>()
                                                .unwrap();
                                            Some(args)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                    let body = read_line!(iter);
                                    defines.insert(
                                        name.to_owned(),
                                        Define {
                                            call: args.is_some(),
                                            args,
                                            statement: body,
                                        },
                                    );
                                } else {
                                    return Err(ArmaConfigError::DefineWithoutName { token: tp });
                                }
                            } else {
                                return Err(ArmaConfigError::DefineWithoutName { token });
                            }
                        }
                        ("undef", true) => {
                            skip_whitespace!(iter);
                            if let Some(tp) = iter.next() {
                                if let Token::Word(name) = tp.token().clone() {
                                    defines.remove(&name);
                                } else {
                                    return Err(ArmaConfigError::UndefineWithoutName { token: tp });
                                }
                            } else {
                                return Err(ArmaConfigError::UndefineWithoutName { token });
                            }
                        }
                        ("ifdef", true) => {
                            skip_whitespace!(iter);
                            if let Some(tp) = iter.next() {
                                if let Token::Word(name) = tp.token().clone() {
                                    if defines.contains_key(&name) {
                                        if_state.push(IfState::ReadingIf);
                                    } else {
                                        if_state.push(IfState::PassingIf);
                                    }
                                }
                            }
                        }
                        ("ifndef", true) => {
                            skip_whitespace!(iter);
                            if let Some(tp) = iter.next() {
                                if let Token::Word(name) = tp.token().clone() {
                                    if defines.contains_key(&name) {
                                        if_state.push(IfState::PassingIf);
                                    } else {
                                        if_state.push(IfState::ReadingIf);
                                    }
                                }
                            }
                        }
                        ("ifdef", false) => {
                            if_state.push(IfState::PassingChild);
                        }
                        ("ifndef", false) => {
                            if_state.push(IfState::PassingChild);
                        }
                        ("else", _) => if_state.flip(),
                        ("endif", _) => {
                            if_state.pop();
                        }
                        ("include", true) => {
                            let file = render(read_line!(iter))
                                .export()
                                .trim_matches('"')
                                .to_owned();
                            ret.append(&mut _preprocess(
                                {
                                    let resolved = resolver.resolve(root, token.path(), &file)?;
                                    super::tokenize(resolved.data(), resolved.path()).unwrap()
                                },
                                root,
                                resolver.clone(),
                                defines,
                            )?);
                        }
                        (_, false) => {
                            read_line!(iter);
                        }
                        (_, true) => {
                            error!(
                                "Unknown directive: {:?} at {}:{}",
                                directive,
                                token.path(),
                                token.start().0
                            );
                            read_line!(iter);
                        }
                    }
                }
            }
            (Token::Word(text), true, _) => {
                if defines.contains_key(text) {
                    ret.append(
                        &mut _resolve(
                            text,
                            &Define {
                                call: false,
                                args: if let Some(tp) = iter.peek() {
                                    if tp.token() == &Token::LeftParenthesis {
                                        Some(
                                            read_args!(iter)
                                                .into_iter()
                                                .map(|arg| {
                                                    _preprocess(
                                                        arg,
                                                        root,
                                                        resolver.clone(),
                                                        defines,
                                                    )
                                                })
                                                .collect::<Result<Vec<Vec<TokenPos>>, ArmaConfigError>>()
                                                .unwrap(),
                                        )
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                },
                                statement: Vec::new(),
                            },
                            root,
                            resolver.clone(),
                            defines,
                        )?
                        .unwrap(),
                    );
                } else {
                    ret.push(token);
                }
            }
            (Token::Newline, true, _) => {
                new_line = true;
                ret.push(token);
            }
            (Token::Whitespace(_), true, _) => {
                ret.push(token);
            }
            (_, true, _) => {
                new_line = false;
                ret.push(token);
            }
            _ => {}
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_args() {
        let tokens = tokenize("(B(C); call f);", "").unwrap();
        let mut a = tokens.into_iter().peekable();
        assert_eq!(
            vec![vec![
                Token::Word(String::from("B")),
                Token::LeftParenthesis,
                Token::Word(String::from("C")),
                Token::RightParenthesis,
                Token::Semicolon,
                Token::Whitespace(Whitespace::Space),
                Token::Word(String::from("call")),
                Token::Whitespace(Whitespace::Space),
                Token::Word(String::from("f"))
            ]],
            vec![read_args!(a)[0]
                .iter()
                .map(|tp| tp.token().to_owned())
                .collect::<Vec<Token>>()]
        )
    }

    #[test]
    fn read_line() {
        let tokens = tokenize("test = false;\ntest = true;\n", "").unwrap();
        let mut a = tokens.into_iter().peekable();
        assert_eq!(
            vec![
                Token::Word(String::from("test")),
                Token::Whitespace(Whitespace::Space),
                Token::Assignment,
                Token::Whitespace(Whitespace::Space),
                Token::Word(String::from("false")),
                Token::Semicolon,
            ],
            read_line!(a)
                .iter()
                .map(|tp| tp.token().to_owned())
                .collect::<Vec<Token>>()
        );

        let tokens = tokenize(" \"\\z\\mod\\addons\"\n", "").unwrap();
        let mut a = tokens.into_iter().peekable();
        assert_eq!(
            vec![
                Token::DoubleQuote,
                Token::Escape,
                Token::Word(String::from("z")),
                Token::Escape,
                Token::Word(String::from("mod")),
                Token::Escape,
                Token::Word(String::from("addons")),
                Token::DoubleQuote
            ],
            read_line!(a)
                .iter()
                .map(|tp| tp.token().to_owned())
                .collect::<Vec<Token>>()
        )
    }
}
