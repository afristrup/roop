use crate::{Doc, print_list, print_type};
use roop_syntax::Param;

pub fn print_params(params: &[Param]) -> Doc {
    let items = params
        .iter()
        .map(|p| Doc::text(format!("{}: {}", p.name, print_type(&p.ty))))
        .collect();
    print_list("(", items, ")", false)
}
