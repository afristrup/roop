use crate::keep_stmt;
use roop_syntax::Block;

/// Lets go of the ancilla `name` where its block ends, like the end of a
/// lifetime.
pub fn release(name: &str, body: &mut Block) {
    let at = body.span.end;
    body.stmts.push(keep_stmt(name, at));
}
