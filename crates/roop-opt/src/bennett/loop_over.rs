use roop_syntax::{BinOp, Block, Expr, Place, Span, Stmt, StmtKind, Type, UpdateOp};

fn stmt(kind: StmtKind) -> Stmt {
    Stmt {
        attrs: Vec::new(),
        kind,
        span: Span::from(0..0),
    }
}

fn var(name: &str) -> Expr {
    Expr::Place(Place::Var(name.to_string()))
}

/// `ancilla i = 0 { from i == 0 { body } loop { i += 1; } until i == len - 1;
/// i -= len - 1; }`: a counted loop that leaves its counter zero, the way the
/// library writes one. `len` is a number or the name of a length parameter.
pub fn loop_over(counter: &str, len: &Result<u64, String>, body: Vec<Stmt>) -> Stmt {
    let last = match len {
        Ok(n) => Expr::Int(*n as i64 - 1),
        Err(name) => Expr::Binary(Box::new(var(name)), BinOp::Sub, Box::new(Expr::Int(1))),
    };
    let update = |op| {
        stmt(StmtKind::Update {
            target: Place::Var(counter.to_string()),
            op,
            value: Expr::Int(1),
        })
    };
    let start = Expr::Binary(Box::new(var(counter)), BinOp::Eq, Box::new(Expr::Int(0)));
    let end = Expr::Binary(Box::new(var(counter)), BinOp::Eq, Box::new(last.clone()));
    let looped = stmt(StmtKind::From {
        entry: start,
        body: Block {
            stmts: body,
            span: Span::from(0..0),
        },
        step: Block {
            stmts: vec![update(UpdateOp::Add)],
            span: Span::from(0..0),
        },
        until: end,
    });
    let rewind = stmt(StmtKind::Update {
        target: Place::Var(counter.to_string()),
        op: UpdateOp::Sub,
        value: last,
    });
    stmt(StmtKind::Ancilla {
        name: counter.to_string(),
        ty: Type::Named("i64".into()),
        init: Expr::Int(0),
        body: Block {
            stmts: vec![looped, rewind],
            span: Span::from(0..0),
        },
    })
}
