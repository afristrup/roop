use crate::{EinsumError, Spec, length_name, loop_over, operand_type, parse_spec};
use roop_syntax::{
    Attr, BinOp, Block, Expr, FnDef, Param, Place, Span, Stmt, StmtKind, Type, UpdateOp,
};

fn stmt(kind: StmtKind) -> Stmt {
    Stmt {
        attrs: Vec::new(),
        kind,
        span: Span::from(0..0),
    }
}

fn counter(label: char) -> String {
    format!("i_{label}")
}

/// `name[i_a][i_b]` for the labels.
fn element(name: &str, labels: &[char]) -> Place {
    labels
        .iter()
        .fold(Place::Var(name.to_string()), |place, l| {
            Place::Index(
                Box::new(place),
                Box::new(Expr::Place(Place::Var(counter(*l)))),
            )
        })
}

/// The loops around `inner`, outermost label first. The first loop over an
/// output label is parallel: each of its iterations writes its own part of the
/// output.
fn nest(labels: &[char], parallel_first: bool, inner: Vec<Stmt>) -> Vec<Stmt> {
    let Some((label, rest)) = labels.split_first() else {
        return inner;
    };
    let body = nest(rest, false, inner);
    let mut looped = loop_over(&counter(*label), &Err(length_name(*label)), body);
    if parallel_first && let StmtKind::Ancilla { body, .. } = &mut looped.kind {
        body.stmts[0].attrs = vec![Attr::Parallel { target: None }];
    }
    vec![looped]
}

/// The function for an `einsum fn`: `out` gets the sum, over the labels that
/// are not in the output, of the product of the elements of the operands
/// `x0`, `x1`, and so on. The sum is added to `out`, so a zero `out` ends up
/// holding the contraction, and running it backward takes it off again.
pub fn build_einsum(def: &FnDef) -> Result<FnDef, EinsumError> {
    let einsum = def.einsum.as_ref().expect("an einsum fn");
    let name = &def.name;
    let Type::Named(elem) = &einsum.elem else {
        return Err(EinsumError::BadType(name.clone()));
    };
    if elem != "i64" && elem != "f64" {
        return Err(EinsumError::BadType(name.clone()));
    }
    let spec: Spec = parse_spec(name, &einsum.spec)?;
    let by_ref = |ty: Type, mutable: bool| Type::Ref {
        mutable,
        inner: Box::new(ty),
    };
    let mut params = vec![Param {
        name: "out".into(),
        ty: by_ref(operand_type(&einsum.elem, &spec.output), true),
    }];
    for (k, labels) in spec.inputs.iter().enumerate() {
        params.push(Param {
            name: format!("x{k}"),
            ty: by_ref(operand_type(&einsum.elem, labels), false),
        });
    }
    let product = spec
        .inputs
        .iter()
        .enumerate()
        .map(|(k, labels)| Expr::Place(element(&format!("x{k}"), labels)))
        .reduce(|a, b| Expr::Binary(Box::new(a), BinOp::Mul, Box::new(b)))
        .expect("at least one operand");
    let update = stmt(StmtKind::Update {
        target: element("out", &spec.output),
        op: UpdateOp::Add,
        value: product,
    });
    let mut order = spec.output.clone();
    order.extend(spec.reduced());
    let body = nest(&order, !spec.output.is_empty(), vec![update]);
    Ok(FnDef {
        name: name.clone(),
        generics: spec.labels().into_iter().map(length_name).collect(),
        params,
        body: Block {
            stmts: body,
            span: Span::from(0..0),
        },
        irreversible: false,
        public: def.public,
        test: false,
        bennett: None,
        external: false,
        world: false,
        einsum: None,
    })
}
