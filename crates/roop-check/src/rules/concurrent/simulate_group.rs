use crate::{ChannelOp, CheckError};
use roop_syntax::Span;
use std::collections::{BTreeMap, BTreeSet};

/// Runs the tasks' channel operations abstractly. Each channel needs at most
/// one sender and one receiver task, so message order is deterministic. Sends
/// never block and a receive waits for a message, so greedy progress decides
/// the outcome: either everything finishes with every channel empty, or the
/// tasks are stuck.
pub fn simulate_group(tasks: &[Vec<ChannelOp>], span: Span) -> Result<(), CheckError> {
    let mut senders: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
    let mut receivers: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
    for (i, ops) in tasks.iter().enumerate() {
        for op in ops {
            let roles = if op.send {
                &mut senders
            } else {
                &mut receivers
            };
            roles.entry(&op.chan).or_default().insert(i);
        }
    }
    for (chan, who) in senders.iter().chain(receivers.iter()) {
        if who.len() > 1 {
            return Err(CheckError::ChannelNotPointToPoint {
                chan: (*chan).into(),
                span,
            });
        }
    }

    let mut queued: BTreeMap<&str, usize> = BTreeMap::new();
    let mut at = vec![0usize; tasks.len()];
    loop {
        let mut progressed = false;
        for (i, ops) in tasks.iter().enumerate() {
            while let Some(op) = ops.get(at[i]) {
                let count = queued.entry(&op.chan).or_default();
                if op.send {
                    *count += 1;
                } else if *count > 0 {
                    *count -= 1;
                } else {
                    break;
                }
                at[i] += 1;
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    let stuck = tasks.iter().enumerate().find_map(|(i, ops)| ops.get(at[i]));
    if let Some(op) = stuck {
        return Err(CheckError::ChannelDeadlock {
            chan: op.chan.clone(),
            span,
        });
    }
    match queued.iter().find(|(_, count)| **count > 0) {
        Some((chan, _)) => Err(CheckError::ChannelNotDrained {
            chan: (*chan).into(),
            span,
        }),
        None => Ok(()),
    }
}
