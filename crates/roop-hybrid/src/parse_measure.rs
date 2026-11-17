use crate::{Cursor, HybridError, Token};

/// The qubit of `measure q` or `measure(q)`, after `measure`.
pub fn parse_measure(c: &mut Cursor) -> Result<String, HybridError> {
    match c.eat(&Token::LParen) {
        true => {
            let qubit = c.name()?;
            c.expect(&Token::RParen)?;
            Ok(qubit)
        }
        false => c.name(),
    }
}
