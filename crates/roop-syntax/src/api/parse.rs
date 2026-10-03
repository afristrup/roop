use crate::{ParseError, Program, parse_spanned};

pub fn parse(src: &str) -> Result<Program, ParseError> {
    let items = parse_spanned(src)?.into_iter().map(|(item, _)| item);
    Ok(Program {
        items: items.collect(),
    })
}
