use crate::{Spec, einsum_fn, einsum_params, einsum_stmt, element, nest, operand_type};
use roop_syntax::{BinOp, Block, Expr, FnDef, Place, Span, StmtKind, Type, UpdateOp};

/// The function of a `q12w` einsum with a sum in it: the exact sum is made by
/// `sum`, one of the `i64` functions, into a scratch array, and its quotient by
/// 4096 is added to `out`, once for each element. The scratch array is taken off
/// again by `uncall`, which the compiler does by setting it to zero.
pub fn build_wide(name: &str, sum: &str, public: bool, spec: &Spec, storage: &Type) -> FnDef {
    let params = einsum_params(spec, storage);
    let args: Vec<Expr> = std::iter::once("w".to_string())
        .chain((0..spec.inputs.len()).map(|k| format!("x{k}")))
        .map(|v| Expr::Place(Place::Var(v)))
        .collect();
    let scaled = einsum_stmt(StmtKind::Update {
        target: element("out", &spec.output),
        op: UpdateOp::Add,
        value: Expr::Binary(
            Box::new(Expr::Place(element("w", &spec.output))),
            BinOp::Div,
            Box::new(Expr::Int(4096)),
        ),
    });
    let call = |uncall: bool| {
        let (callee, generics, args) = (sum.to_string(), Vec::new(), args.clone());
        einsum_stmt(match uncall {
            true => StmtKind::Uncall {
                callee,
                generics,
                args,
            },
            false => StmtKind::Call {
                callee,
                generics,
                args,
            },
        })
    };
    let mut stmts = vec![call(false)];
    stmts.extend(nest(&spec.output, false, vec![scaled]));
    stmts.push(call(true));
    let scratch = einsum_stmt(StmtKind::Ancilla {
        name: "w".into(),
        ty: operand_type(storage, &spec.output),
        init: Expr::Int(0),
        body: Block {
            stmts,
            span: Span::from(0..0),
        },
    });
    einsum_fn(name, public, spec, params, vec![scratch])
}
