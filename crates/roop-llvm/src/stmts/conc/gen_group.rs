use crate::{CodegenError, Dir, FnGen, capture_env, check_abort, outline_block};
use roop_syntax::{Stmt, StmtKind};

/// Runs adjacent `#[concurrent]` blocks as tasks on their own threads and
/// waits for all of them. Backward, each task runs backward, so every send
/// becomes a receive and the messages flow the other way.
pub fn gen_group(g: &mut FnGen, tasks: &[&Stmt], dir: Dir) -> Result<(), CodegenError> {
    let env = capture_env(g);
    let mut symbols = Vec::new();
    for task in tasks {
        let StmtKind::Block(body) = &task.kind else {
            return Err(CodegenError::InvalidOperand("#[concurrent] needs a block"));
        };
        symbols.push(outline_block(g, None, body, dir, "task")?);
    }
    let n = tasks.len();
    let (table, envs) = (
        g.alloca(&format!("[{n} x ptr]")),
        g.alloca(&format!("[{n} x ptr]")),
    );
    for (i, symbol) in symbols.iter().enumerate() {
        for (array, value) in [(&table, format!("@{symbol}")), (&envs, env.clone())] {
            let at = format!("%{}", g.fresh("t"));
            g.emit(&format!(
                "{at} = getelementptr inbounds [{n} x ptr], ptr {array}, i64 0, i64 {i}"
            ));
            g.emit(&format!("store ptr {value}, ptr {at}"));
        }
    }
    g.emit(&format!(
        "call void @roop_concurrent(i64 {n}, ptr {table}, ptr {envs})"
    ));
    check_abort(g)
}
