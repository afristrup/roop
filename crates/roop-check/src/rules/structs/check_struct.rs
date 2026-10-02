use crate::{CheckError, Scope, check_block, contains_irrev, inverts};
use roop_syntax::StructDef;

/// `build` and `unbuild` come as a pair, and `unbuild` must undo `build`
/// statement by statement in reverse order over the same parameters.
pub fn check_struct(def: &StructDef, scope: Scope) -> Result<(), CheckError> {
    let (build, unbuild) = match (&def.build, &def.unbuild) {
        (None, None) => return Ok(()),
        (Some(build), Some(unbuild)) => (build, unbuild),
        _ => {
            return Err(CheckError::UnpairedBuild {
                name: def.name.clone(),
                span: def.span,
            });
        }
    };
    let reversible = Scope {
        irrev: false,
        ..scope
    };
    for ctor in [build, unbuild] {
        if contains_irrev(&ctor.body) {
            return Err(CheckError::IrreversibleOutsideIrrev { span: def.span });
        }
        check_block(&ctor.body, reversible)?;
    }
    let inverse = build.params == unbuild.params
        && build.body.stmts.len() == unbuild.body.stmts.len()
        && build
            .body
            .stmts
            .iter()
            .zip(unbuild.body.stmts.iter().rev())
            .all(|(b, u)| inverts(&b.kind, &u.kind));
    if inverse {
        Ok(())
    } else {
        Err(CheckError::UnbuildNotInverse {
            name: def.name.clone(),
            span: def.span,
        })
    }
}
