use roop_syntax::UpdateOp;

pub fn update_text(op: UpdateOp) -> &'static str {
    match op {
        UpdateOp::Add => "+=",
        UpdateOp::Sub => "-=",
        UpdateOp::Xor => "^=",
        UpdateOp::Mul => "*=",
        UpdateOp::Div => "/=",
    }
}
