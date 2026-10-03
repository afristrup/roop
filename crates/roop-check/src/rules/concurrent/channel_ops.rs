use crate::{ChannelOp, CheckError, child_blocks, is_task};
use roop_syntax::{Block, StmtKind};

/// The channel operations of one task, in order, for channels declared
/// outside it. Operations that are not on a fixed straight-line path make the
/// protocol unknowable, so they are errors. That includes `try`: rolling back
/// half of a conversation would break the synchrony the peer relies on.
pub fn channel_ops(block: &Block, inner: &mut Vec<String>) -> Result<Vec<ChannelOp>, CheckError> {
    let mut ops = Vec::new();
    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Send { chan, .. } | StmtKind::Recv { chan, .. } => {
                if !inner.contains(chan) {
                    let send = matches!(stmt.kind, StmtKind::Send { .. });
                    ops.push(ChannelOp {
                        chan: chan.clone(),
                        send,
                    });
                }
            }
            StmtKind::Chan { name, body, .. } => {
                inner.push(name.clone());
                ops.extend(channel_ops(body, inner)?);
                inner.pop();
            }
            StmtKind::Ancilla { body, .. } | StmtKind::Borrow { body, .. } if !is_task(stmt) => {
                ops.extend(channel_ops(body, inner)?);
            }
            StmtKind::Block(body) if !is_task(stmt) => ops.extend(channel_ops(body, inner)?),
            _ => {
                for child in child_blocks(stmt) {
                    if let Some(op) = channel_ops(child, inner)?.first() {
                        return Err(CheckError::ChannelOpInControlFlow {
                            chan: op.chan.clone(),
                            span: stmt.span,
                        });
                    }
                }
            }
        }
    }
    Ok(ops)
}
