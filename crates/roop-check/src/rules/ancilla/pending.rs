use std::collections::HashSet;

pub struct Pending<'a> {
    pub stmt: &'a roop_syntax::Stmt,
    pub reads: HashSet<&'a str>,
    pub tainted: bool,
}
