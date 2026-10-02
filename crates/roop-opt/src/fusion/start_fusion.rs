use crate::{Fused, par_loop};
use roop_syntax::Stmt;

pub fn start_fusion(stmt: &Stmt) -> Option<Fused> {
    par_loop(stmt)?;
    Some(Fused {
        head: stmt.clone(),
        prologue: Vec::new(),
        epilogue: Vec::new(),
    })
}
