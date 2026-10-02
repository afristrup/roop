/// `use a::b::c;` or `use a::b::c as d;`: brings one item into scope.
#[derive(Clone, Debug, PartialEq)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub alias: Option<String>,
}
