use crate::{Spec, einsum_fn, einsum_params, einsum_stmt, element, nest};
use roop_syntax::{BinOp, Expr, FnDef, StmtKind, Type, UpdateOp};

/// The function that adds into `out` the sum, over the labels that are not in
/// the output, of the product of the elements of the operands `x0`, `x1`, and so
/// on. With `scaled` each product is divided by 4096 first, as in `q12`. The
/// loops over the output labels are parallel, and the sum is in the order of
/// the loops, so it is the same on every run.
pub fn build_sum(name: &str, public: bool, spec: &Spec, storage: &Type, scaled: bool) -> FnDef {
    let product = spec
        .inputs
        .iter()
        .enumerate()
        .map(|(k, labels)| Expr::Place(element(&format!("x{k}"), labels)))
        .reduce(|a, b| {
            let product = Expr::Binary(Box::new(a), BinOp::Mul, Box::new(b));
            match scaled {
                true => Expr::Binary(Box::new(product), BinOp::Div, Box::new(Expr::Int(4096))),
                false => product,
            }
        })
        .expect("at least one operand");
    let update = einsum_stmt(StmtKind::Update {
        target: element("out", &spec.output),
        op: UpdateOp::Add,
        value: product,
    });
    let mut order = spec.output.clone();
    order.extend(spec.reduced());
    let body = nest(&order, !spec.output.is_empty(), vec![update]);
    einsum_fn(name, public, spec, einsum_params(spec, storage), body)
}
