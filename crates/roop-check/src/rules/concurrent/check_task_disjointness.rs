use crate::{Access, CheckError, collect_accesses, place_root, places_overlap};
use roop_syntax::{Block, Span};

/// Tasks share no mutable state: a place one task writes may not be touched by
/// any other task. Reading shared data is fine.
pub fn check_task_disjointness(tasks: &[&Block], span: Span) -> Result<(), CheckError> {
    let accesses: Vec<Vec<Access>> = tasks
        .iter()
        .map(|task| {
            let mut out = Vec::new();
            collect_accesses(task, &mut Vec::new(), &mut out, None);
            out
        })
        .collect();
    for (i, mine) in accesses.iter().enumerate() {
        for write in mine.iter().filter(|a| a.write && !a.local) {
            for (j, theirs) in accesses.iter().enumerate() {
                let clash = j != i
                    && theirs
                        .iter()
                        .any(|a| !a.local && places_overlap(&write.place, &a.place));
                if clash {
                    let var = place_root(&write.place).into();
                    return Err(CheckError::ConcurrentConflict { var, span });
                }
            }
        }
    }
    Ok(())
}
