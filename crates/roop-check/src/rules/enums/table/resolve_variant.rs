use crate::{CheckError, Enums};
use roop_syntax::Span;

pub fn resolve_variant(
    enums: &Enums,
    enum_name: &str,
    variant: &str,
    span: Span,
) -> Result<(), CheckError> {
    let known = enums
        .get(enum_name)
        .is_some_and(|def| def.variants.iter().any(|v| v == variant));
    if known {
        Ok(())
    } else {
        Err(CheckError::UnknownVariant {
            enum_name: enum_name.into(),
            variant: variant.into(),
            span,
        })
    }
}
