use crate::LeanError;
use roop_syntax::OverwriteOp;

/// The value `old op e` an overwrite stores; plain assignment is just `e`.
pub fn overwrite_value(
    op: OverwriteOp,
    float: bool,
    old: &str,
    e: &str,
) -> Result<String, LeanError> {
    Ok(match (op, float) {
        (OverwriteOp::Assign, _) => e.to_string(),
        (OverwriteOp::Rem, false) => format!("(BitVec.srem {old} {e})"),
        (OverwriteOp::Rem, true) => {
            return Err(LeanError::Unsupported("remainder of floats".into()));
        }
    })
}
