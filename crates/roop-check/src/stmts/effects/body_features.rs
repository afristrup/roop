use crate::{child_blocks, stmt_exprs};
use roop_syntax::{Block, Expr, Place, Stmt, StmtKind};

/// A rough static profile of a block, used to estimate where it runs fastest.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BodyFeatures {
    /// Operations per execution: one per expression node, two per update.
    pub work: u64,
    pub has_call: bool,
    pub has_float_literal: bool,
}

pub fn body_features(block: &Block) -> BodyFeatures {
    let mut features = BodyFeatures::default();
    visit_block(block, &mut features);
    features
}

fn visit_block(block: &Block, f: &mut BodyFeatures) {
    for stmt in &block.stmts {
        visit_stmt(stmt, f);
    }
}

fn visit_stmt(stmt: &Stmt, f: &mut BodyFeatures) {
    f.work += match &stmt.kind {
        StmtKind::Update { .. } | StmtKind::Swap(..) => 2,
        _ => 1,
    };
    if matches!(stmt.kind, StmtKind::Call { .. } | StmtKind::Uncall { .. }) {
        f.has_call = true;
    }
    for expr in stmt_exprs(stmt) {
        visit_expr(expr, f);
    }
    for child in child_blocks(stmt) {
        visit_block(child, f);
    }
}

fn visit_expr(expr: &Expr, f: &mut BodyFeatures) {
    f.work += 1;
    match expr {
        Expr::Float(_) => f.has_float_literal = true,
        Expr::Unary(_, inner) => visit_expr(inner, f),
        Expr::Binary(l, _, r) => {
            visit_expr(l, f);
            visit_expr(r, f);
        }
        Expr::Place(place) => visit_place(place, f),
        Expr::Int(_) | Expr::Bool(_) | Expr::Variant(..) => {}
    }
}

fn visit_place(place: &Place, f: &mut BodyFeatures) {
    match place {
        Place::Var(_) => {}
        Place::Field(base, _) => visit_place(base, f),
        Place::Index(base, index) => {
            visit_place(base, f);
            visit_expr(index, f);
        }
    }
}
