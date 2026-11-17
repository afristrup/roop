use crate::{
    Cursor, HybridError, Kind, Stmt, Token, gate_word, parse_block, parse_expr, parse_gate,
    parse_measure,
};

/// One statement. `do p while b` comes back as `p; while b do p done`.
pub fn parse_stmt(c: &mut Cursor) -> Result<Vec<Stmt>, HybridError> {
    let line = c.line();
    let word = match c.peek() {
        Token::Ident(word) => word.clone(),
        _ => return Err(c.error("a statement")),
    };
    let stmt = match word.as_str() {
        "skip" => {
            c.bump();
            Stmt::Skip
        }
        "qubit" | "bit" | "int" => {
            c.bump();
            let kind = match word.as_str() {
                "qubit" => Kind::Qubit,
                "bit" => Kind::Bit,
                _ => Kind::Int,
            };
            Stmt::Declare {
                kind,
                name: c.name()?,
                line,
            }
        }
        "if" => {
            c.bump();
            let cond = parse_expr(c, 0)?;
            c.expect_word("then")?;
            let then = parse_block(c, &["else", "end"])?;
            let els = match c.eat_word("else") {
                true => parse_block(c, &["end"])?,
                false => Vec::new(),
            };
            c.expect_word("end")?;
            Stmt::If {
                cond,
                then,
                els,
                line,
            }
        }
        "while" => {
            c.bump();
            let cond = parse_expr(c, 0)?;
            c.expect_word("do")?;
            let body = parse_block(c, &["done"])?;
            c.expect_word("done")?;
            Stmt::While { cond, body, line }
        }
        "do" => {
            c.bump();
            let body = parse_block(c, &["while"])?;
            c.expect_word("while")?;
            let cond = parse_expr(c, 0)?;
            let mut stmts = body.clone();
            stmts.push(Stmt::While { cond, body, line });
            return Ok(stmts);
        }
        _ if gate_word(&word).is_some() || word == "R" => parse_gate(c, &word)?,
        _ => {
            let name = c.name()?;
            c.expect(&Token::Assign)?;
            match c.eat_word("measure") {
                true => Stmt::Measure {
                    bit: name,
                    qubit: parse_measure(c)?,
                    line,
                },
                false => Stmt::Assign {
                    name,
                    value: parse_expr(c, 0)?,
                    line,
                },
            }
        }
    };
    Ok(vec![stmt])
}
