use crate::{BennettError, copy_stmts};
use roop_syntax::{Block, Expr, FnDef, Param, Place, Program, Span, Stmt, StmtKind, Type};

fn stmt(kind: StmtKind) -> Stmt {
    Stmt {
        attrs: Vec::new(),
        kind,
        span: Span::from(0..0),
    }
}

fn is_history(ty: &Type) -> bool {
    matches!(ty, Type::Stack(..) | Type::Param { stack: true, .. })
}

/// The Bennett version of `target`: it runs `target` with its history stacks
/// as local ancillas, copies what the mutable parameters ended up as into new
/// zero parameters `<name>_out`, and uncomputes. The inputs come back and the
/// histories are empty again, so the garbage is gone and what remains depends
/// only on what `target` computes.
pub fn build_bennett(
    program: &Program,
    name: &str,
    public: bool,
    target: &FnDef,
) -> Result<FnDef, BennettError> {
    let err = |param: &str| BennettError::Uncopyable {
        name: name.into(),
        param: param.into(),
    };
    if target.irreversible {
        return Err(BennettError::NotReversible {
            name: name.into(),
            target: target.name.clone(),
        });
    }
    if target.bennett.is_some() {
        return Err(BennettError::Chained {
            name: name.into(),
            target: target.name.clone(),
        });
    }
    let mut params = Vec::new();
    let mut outs = Vec::new();
    let mut histories = Vec::new();
    let mut copies = Vec::new();
    for param in &target.params {
        let Type::Ref { mutable, inner } = &param.ty else {
            return Err(err(&param.name));
        };
        if is_history(inner) {
            histories.push((param.name.clone(), (**inner).clone()));
            continue;
        }
        params.push(param.clone());
        if !*mutable {
            continue;
        }
        let out_name = format!("{}_out", param.name);
        if target.params.iter().any(|p| p.name == out_name) {
            return Err(BennettError::NameTaken {
                name: name.into(),
                param: param.name.clone(),
            });
        }
        let out = Place::Var(out_name.clone());
        let from = Place::Var(param.name.clone());
        copies.extend(copy_stmts(program, &out, &from, inner, 0).ok_or_else(|| err(&param.name))?);
        outs.push(Param {
            name: out_name,
            ty: param.ty.clone(),
        });
    }
    params.extend(outs);
    let var = |n: &str| Expr::Place(Place::Var(n.to_string()));
    let call = |uncall: bool| {
        let generics = target.generics.iter().map(|g| var(g)).collect();
        let args = target.params.iter().map(|p| var(&p.name)).collect();
        let callee = target.name.clone();
        stmt(if uncall {
            StmtKind::Uncall {
                callee,
                generics,
                args,
            }
        } else {
            StmtKind::Call {
                callee,
                generics,
                args,
            }
        })
    };
    let mut body = vec![call(false)];
    body.extend(copies);
    body.push(call(true));
    for (history, ty) in histories.into_iter().rev() {
        body = vec![stmt(StmtKind::Ancilla {
            name: history,
            ty,
            init: Expr::Empty,
            body: Block {
                stmts: body,
                span: Span::from(0..0),
            },
        })];
    }
    Ok(FnDef {
        name: name.to_string(),
        generics: target.generics.clone(),
        params,
        body: Block {
            stmts: body,
            span: Span::from(0..0),
        },
        irreversible: false,
        public,
        test: false,
        bennett: None,
    })
}
