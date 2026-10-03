use roop_syntax::{Block, Stmt, StmtKind};

/// The statements of `block`, looking through nested ancilla blocks. An
/// ancilla block only scopes a variable and always runs its body, so for the
/// enclosing ancilla `name` its statements are as good as straight-line. A
/// nested ancilla that reuses `name` shadows it and stays opaque.
pub fn flatten_ancillas<'a>(block: &'a Block, name: &str) -> Vec<&'a Stmt> {
    let mut out = Vec::new();
    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Ancilla {
                name: inner, body, ..
            } if inner != name => {
                out.extend(flatten_ancillas(body, name));
            }
            _ => out.push(stmt),
        }
    }
    out
}
