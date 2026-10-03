use crate::{CheckError, Enums, expr_variants, resolve_variant, stmt_exprs};
use roop_syntax::{Pattern, Stmt, StmtKind};

/// Every `Enum::Variant` in expressions and match patterns must exist.
pub fn check_variants(enums: &Enums, stmt: &Stmt) -> Result<(), CheckError> {
    let mut used = Vec::new();
    for expr in stmt_exprs(stmt) {
        expr_variants(expr, &mut used);
    }
    if let StmtKind::Match { arms, .. } = &stmt.kind {
        for arm in arms {
            if let Pattern::Variant(e, v) = &arm.pattern {
                used.push((e, v));
            }
        }
    }
    used.into_iter()
        .try_for_each(|(e, v)| resolve_variant(enums, e, v, stmt.span))
}
