use crate::{Doc, escape, float_text, print_binary, print_place, print_type};
use roop_syntax::{Expr, UnOp};

pub fn print_expr(expr: &Expr) -> Doc {
    match expr {
        Expr::Int(n) => Doc::text(n.to_string()),
        Expr::Float(x) => Doc::text(float_text(*x)),
        Expr::Bool(b) => Doc::text(b.to_string()),
        Expr::Byte(b) => Doc::text(format!("b'{}'", escape(&[*b]))),
        Expr::Str(bytes) => Doc::text(format!("\"{}\"", escape(bytes))),
        Expr::Empty => Doc::text("empty"),
        Expr::Variant(e, v) => Doc::text(format!("{e}::{v}")),
        Expr::Place(place) => print_place(place),
        Expr::Unary(op, inner) => {
            let sign = match op {
                UnOp::Neg => "-",
                UnOp::Not => "!",
            };
            let operand = match **inner {
                Expr::Binary(..) => parenthesized(print_expr(inner)),
                _ => print_expr(inner),
            };
            Doc::concat(vec![Doc::text(sign), operand])
        }
        Expr::Cast(inner, ty) => {
            let operand = match **inner {
                Expr::Binary(..) => parenthesized(print_expr(inner)),
                _ => print_expr(inner),
            };
            Doc::concat(vec![operand, Doc::text(format!(" as {}", print_type(ty)))])
        }
        Expr::Binary(..) => print_binary(expr),
    }
}

pub fn parenthesized(doc: Doc) -> Doc {
    Doc::concat(vec![Doc::text("("), doc, Doc::text(")")])
}
