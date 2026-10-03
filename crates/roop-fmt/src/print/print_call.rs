use crate::{Doc, print_expr, print_generics, print_list};
use roop_syntax::Expr;

/// `call f<8>(a, b);` or the same with `uncall`.
pub fn print_call(keyword: &str, callee: &str, generics: &[Expr], args: &[Expr]) -> Doc {
    Doc::concat(vec![
        Doc::text(format!("{keyword} {callee}")),
        print_generics(generics.iter().map(print_expr).collect()),
        print_list("(", args.iter().map(print_expr).collect(), ")", false),
        Doc::text(";"),
    ])
}
