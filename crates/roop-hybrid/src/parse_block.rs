use crate::{Cursor, HybridError, Stmt, Token, parse_stmt};

/// Statements separated by `;` up to one of the words in `ends` or the end of
/// the file. A `;` before the end is allowed.
pub fn parse_block(c: &mut Cursor, ends: &[&str]) -> Result<Vec<Stmt>, HybridError> {
    let at_end = |c: &Cursor| *c.peek() == Token::Eof || ends.iter().any(|w| c.at_word(w));
    let mut stmts = Vec::new();
    while !at_end(c) {
        stmts.extend(parse_stmt(c)?);
        if !c.eat(&Token::Semi) && !at_end(c) {
            return Err(c.error("`;`"));
        }
    }
    Ok(stmts)
}
