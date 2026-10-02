use roop_syntax::{StmtKind, UpdateOp};

pub fn inverts(a: &StmtKind, b: &StmtKind) -> bool {
    match (a, b) {
        (
            StmtKind::Update {
                target: ta,
                op: oa,
                value: va,
            },
            StmtKind::Update {
                target: tb,
                op: ob,
                value: vb,
            },
        ) => ta == tb && va == vb && opposite(*oa) == *ob,
        (StmtKind::Swap(a1, a2), StmtKind::Swap(b1, b2)) => {
            (a1 == b1 && a2 == b2) || (a1 == b2 && a2 == b1)
        }
        (
            StmtKind::Call {
                callee: ca,
                args: aa,
            },
            StmtKind::Uncall {
                callee: cb,
                args: ab,
            },
        )
        | (
            StmtKind::Uncall {
                callee: ca,
                args: aa,
            },
            StmtKind::Call {
                callee: cb,
                args: ab,
            },
        ) => ca == cb && aa == ab,
        _ => false,
    }
}

fn opposite(op: UpdateOp) -> UpdateOp {
    match op {
        UpdateOp::Add => UpdateOp::Sub,
        UpdateOp::Sub => UpdateOp::Add,
        UpdateOp::Xor => UpdateOp::Xor,
    }
}
