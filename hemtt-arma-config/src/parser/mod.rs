use std::{collections::HashMap, io::Write};

use pest::Parser;

mod node;
pub use node::Node;

mod statement;
pub use statement::Statement;

#[derive(Parser)]
#[grammar = "parser/config.pest"]
pub struct ConfigParser;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
/// Abstract Syntax Tree
pub struct AST {
    pub config: Node,
}

// impl AST {
//     pub fn valid(&self) -> bool {
//         if let Some(report) = &self.report {
//             report.errors.is_empty()
//         } else {
//             true
//         }
//     }
// }

/// Converts a raw string into an AST
///
/// ```
/// let content = "value = 123;";
/// hemtt_arma_config::parse(content, "doc test", None);
/// ```
pub fn parse(
    source: &str,
    context: &str,
    map: Option<HashMap<usize, crate::preprocess::LineMap>>,
) -> Result<AST, String> {
    let clean = source.replace('\r', "");
    let pair = ConfigParser::parse(Rule::file, &clean)
        .unwrap_or_else(|e| {
            let out = std::env::temp_dir().join("failed.txt");
            let mut f = std::fs::File::create(&out).expect("failed to create failed.txt");
            f.write_all(clean.as_bytes()).unwrap();
            f.flush().unwrap();
            if let Some(map) = map {
                let (line, col) = match e.line_col {
                    pest::error::LineColLocation::Pos(s) => s,
                    pest::error::LineColLocation::Span(s, _) => s,
                };
                let spans = map.get(&line).unwrap();
                if let Some(span) = spans.iter().find(|s| s.0 <= col && s.0 + s.1 >= col) {
                    panic!(
                        "failed to parse context: {}, {} ({}:{})\nerror: {}",
                        context, span.2, span.3 .0, span.3 .1, e,
                    )
                }
            }
            panic!(
                "failed to parse context: {}, saved at {}. error: {}",
                context,
                out.display(),
                e
            )
        })
        .next()
        .unwrap();
    let pair = pair.into_inner().next().unwrap();
    let config = Node::from_expr(std::env::current_dir().unwrap(), source, pair)?;
    Ok(AST { config })
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn property() {
        let ast = parse("value = 123;", "test", None);
        println!("{:?}", ast);
    }

    #[test]
    fn hex() {
        let ast = parse("value = 0x123;", "test", None);
        println!("{:?}", ast);
    }
}
