use crate::{HybridError, Kind, Signature};

/// Fails unless `name` is declared as `expected`.
pub fn lookup(sig: &Signature, name: &str, expected: Kind, line: usize) -> Result<(), HybridError> {
    match sig.get(name) {
        None => Err(HybridError::Undeclared {
            line,
            name: name.into(),
        }),
        Some(found) if found != expected => Err(HybridError::Kind {
            line,
            name: name.into(),
            expected,
            found,
        }),
        Some(_) => Ok(()),
    }
}
