mod build;
mod lookup;

pub use build::*;
pub use lookup::*;

#[derive(Debug, Default)]
pub struct Index {
    pub(crate) file: String,
    pub(crate) inputs: usize,
    pub node_count: usize,
    pub(crate) nodes: Vec<Node>,
}

#[derive(Debug, Default)]
pub(crate) struct Node {
    pub(crate) zero: Option<usize>,
    pub(crate) one: Option<usize>,
    pub(crate) any: Option<usize>,
    pub(crate) outputs: Vec<String>,
}
