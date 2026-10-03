use crate::{CliError, Fixture};
use roop_syntax::{Param, Type};

fn flat(ty: &Type) -> Option<(char, usize)> {
    match ty {
        Type::Named(name) => match name.as_str() {
            "i64" => Some(('i', 1)),
            "f64" => Some(('f', 1)),
            "bool" => Some(('b', 1)),
            "u8" => Some(('u', 1)),
            _ => None,
        },
        Type::Array(elem, len) => {
            let (kind, count) = flat(elem)?;
            Some((kind, count * *len as usize))
        }
        Type::Stack(elem, cap) if matches!(**elem, Type::Named(ref n) if n == "i64") => {
            Some(('i', 1 + *cap as usize))
        }
        _ => None,
    }
}

/// The fixture a test parameter stands for.
pub fn fixture_of(test: &str, param: &Param) -> Result<Fixture, CliError> {
    let Type::Ref { inner, .. } = &param.ty else {
        return Err(CliError::Tool(format!(
            "test {test}: fixture {} is not a reference",
            param.name
        )));
    };
    let (kind, count) = flat(inner).ok_or_else(|| {
        CliError::Tool(format!(
            "test {test}: fixture {} must be made of i64, f64, u8, bool, arrays of them, or a stack of i64",
            param.name
        ))
    })?;
    Ok(Fixture {
        name: param.name.clone(),
        kind,
        count,
    })
}
