use crate::CodegenError;
use roop_syntax::Type;

pub fn same_type(expected: &Type, found: &Type) -> Result<(), CodegenError> {
    if expected == found {
        Ok(())
    } else {
        Err(CodegenError::TypeMismatch {
            expected: format!("{expected:?}"),
            found: format!("{found:?}"),
        })
    }
}
