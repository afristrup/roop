use crate::{child_blocks, stmt_exprs};
use roop_syntax::{Block, Expr, Place, Stmt, StmtKind};
use std::collections::BTreeMap;

/// A rough static profile of a block, used to estimate where it runs fastest.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BodyFeatures {
    /// Operations per execution: one per expression node, two per update.
    pub work: u64,
    /// Bytes of array data one execution moves: every distinct element it
    /// touches counts once, twice when written (read and write back), at 8
    /// bytes each. Repeated touches of one element are cached.
    pub bytes: u64,
    pub has_call: bool,
    pub has_float_literal: bool,
}

pub fn body_features(block: &Block) -> BodyFeatures {
    let mut features = BodyFeatures::default();
    let mut cells = BTreeMap::new();
    visit_block(block, &mut features, &mut cells);
    features.bytes = cells
        .values()
        .map(|written| 8 * (1 + u64::from(*written)))
        .sum();
    features
}

type Cells = BTreeMap<String, bool>;

fn visit_block(block: &Block, f: &mut BodyFeatures, cells: &mut Cells) {
    for stmt in &block.stmts {
        visit_stmt(stmt, f, cells);
    }
}

fn visit_stmt(stmt: &Stmt, f: &mut BodyFeatures, cells: &mut Cells) {
    f.work += match &stmt.kind {
        StmtKind::Update { .. } | StmtKind::Overwrite { .. } | StmtKind::Swap(..) => 2,
        _ => 1,
    };
    match &stmt.kind {
        StmtKind::Update { target, .. } | StmtKind::Overwrite { target, .. } => {
            touch(target, true, f, cells)
        }
        StmtKind::Swap(a, b) => {
            touch(a, true, f, cells);
            touch(b, true, f, cells);
        }
        StmtKind::Call { .. } | StmtKind::Uncall { .. } => f.has_call = true,
        _ => {}
    }
    for expr in stmt_exprs(stmt) {
        visit_expr(expr, f, cells);
    }
    for child in child_blocks(stmt) {
        visit_block(child, f, cells);
    }
}

fn visit_expr(expr: &Expr, f: &mut BodyFeatures, cells: &mut Cells) {
    f.work += 1;
    match expr {
        Expr::Float(_) => f.has_float_literal = true,
        Expr::Unary(_, inner) => visit_expr(inner, f, cells),
        Expr::Binary(l, _, r) => {
            visit_expr(l, f, cells);
            visit_expr(r, f, cells);
        }
        Expr::Place(place) => touch(place, false, f, cells),
        Expr::Int(_) | Expr::Bool(_) | Expr::Variant(..) => {}
    }
}

/// Records an access to an array element, and visits the index expressions.
fn touch(place: &Place, write: bool, f: &mut BodyFeatures, cells: &mut Cells) {
    if has_index(place) {
        *cells.entry(format!("{place:?}")).or_default() |= write;
    }
    visit_indices(place, f, cells);
}

fn visit_indices(place: &Place, f: &mut BodyFeatures, cells: &mut Cells) {
    match place {
        Place::Var(_) => {}
        Place::Field(base, _) => visit_indices(base, f, cells),
        Place::Index(base, index) => {
            visit_indices(base, f, cells);
            visit_expr(index, f, cells);
        }
    }
}

fn has_index(place: &Place) -> bool {
    match place {
        Place::Var(_) => false,
        Place::Field(base, _) => has_index(base),
        Place::Index(..) => true,
    }
}
