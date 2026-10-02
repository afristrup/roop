use roop_syntax::StmtKind;

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
        ) => ta == tb && va == vb && oa.inverse() == *ob,
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
        (
            StmtKind::Send {
                chan: ca,
                source: pa,
            },
            StmtKind::Recv {
                chan: cb,
                target: pb,
            },
        )
        | (
            StmtKind::Recv {
                chan: ca,
                target: pa,
            },
            StmtKind::Send {
                chan: cb,
                source: pb,
            },
        ) => ca == cb && pa == pb,
        (
            StmtKind::Push {
                stack: sa,
                source: pa,
            },
            StmtKind::Pop {
                stack: sb,
                target: pb,
            },
        )
        | (
            StmtKind::Pop {
                stack: sa,
                target: pa,
            },
            StmtKind::Push {
                stack: sb,
                source: pb,
            },
        ) => sa == sb && pa == pb,
        _ => false,
    }
}
