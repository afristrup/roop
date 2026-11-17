use crate::{Cursor, HybridError, Stmt, Token, lex, parse_block};

/// The statements of an HQbricks∞ program.
pub fn parse(text: &str) -> Result<Vec<Stmt>, HybridError> {
    let mut c = Cursor::new(lex(text)?);
    let stmts = parse_block(&mut c, &[])?;
    c.expect(&Token::Eof)?;
    Ok(stmts)
}
