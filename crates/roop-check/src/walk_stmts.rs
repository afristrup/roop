use super::{CheckError, child_blocks};
use roop_syntax::{Block, Stmt};

pub fn walk_stmts<'a, F>(block: &'a Block, f: &mut F) -> Result<(), CheckError>
where
    F: FnMut(&'a Stmt) -> Result<(), CheckError>,
{
    for stmt in &block.stmts {
        f(stmt)?;
        for child in child_blocks(stmt) {
            walk_stmts(child, f)?;
        }
    }
    Ok(())
}
