use crate::loop_over;
use roop_syntax::{Expr, Item, Place, Program, Span, Stmt, StmtKind, Type, UpdateOp};

fn stmt(kind: StmtKind) -> Stmt {
    Stmt {
        attrs: Vec::new(),
        kind,
        span: Span::from(0..0),
    }
}

/// Statements that add `from` into the zero `out`, so `out` ends equal to it.
/// None when the type is not one that can be copied this way.
pub fn copy_stmts(
    program: &Program,
    out: &Place,
    from: &Place,
    ty: &Type,
    depth: usize,
) -> Option<Vec<Stmt>> {
    let update = |op| {
        vec![stmt(StmtKind::Update {
            target: out.clone(),
            op,
            value: Expr::Place(from.clone()),
        })]
    };
    match ty {
        Type::Named(name) => match name.as_str() {
            "i64" | "f64" => Some(update(UpdateOp::Add)),
            "bool" => Some(update(UpdateOp::Xor)),
            _ => {
                let def = program.items.iter().find_map(|item| match item {
                    Item::Struct(s) if s.name == *name => Some(s),
                    _ => None,
                })?;
                let mut all = Vec::new();
                for field in &def.fields {
                    let o = Place::Field(Box::new(out.clone()), field.name.clone());
                    let f = Place::Field(Box::new(from.clone()), field.name.clone());
                    all.extend(copy_stmts(program, &o, &f, &field.ty, depth)?);
                }
                Some(all)
            }
        },
        Type::Array(elem, len) => elements(program, out, from, elem, Ok(*len), depth),
        Type::Param {
            elem,
            len,
            stack: false,
        } => elements(program, out, from, elem, Err(len.clone()), depth),
        _ => None,
    }
}

fn elements(
    program: &Program,
    out: &Place,
    from: &Place,
    elem: &Type,
    len: Result<u64, String>,
    depth: usize,
) -> Option<Vec<Stmt>> {
    let counter = format!("__b{depth}");
    let at = |p: &Place| {
        Place::Index(
            Box::new(p.clone()),
            Box::new(Expr::Place(Place::Var(counter.clone()))),
        )
    };
    let body = copy_stmts(program, &at(out), &at(from), elem, depth + 1)?;
    Some(vec![loop_over(&counter, &len, body)])
}
