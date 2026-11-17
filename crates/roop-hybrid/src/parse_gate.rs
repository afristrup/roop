use crate::{Cursor, GateName, HybridError, Stmt, Token, gate_word};

/// `G(q, ...)`, or `R(q, k)` for R_k.
pub fn parse_gate(c: &mut Cursor, word: &str) -> Result<Stmt, HybridError> {
    let line = c.line();
    c.bump();
    c.expect(&Token::LParen)?;
    let mut qubits = vec![c.name()?];
    let mut k = None;
    while c.eat(&Token::Comma) {
        match (word, c.peek().clone()) {
            ("R", Token::Num(n)) => {
                c.bump();
                k = Some(n);
                break;
            }
            _ => qubits.push(c.name()?),
        }
    }
    c.expect(&Token::RParen)?;
    let gate = match (word, k) {
        ("R", Some(k)) if (0..=62).contains(&k) => GateName::R(k as u32),
        ("R", Some(k)) => return Err(HybridError::Rotation { line, k }),
        ("R", None) => return Err(c.error("`, k` for R_k")),
        _ => gate_word(word).ok_or_else(|| c.error("a gate"))?,
    };
    Ok(Stmt::Gate { gate, qubits, line })
}
