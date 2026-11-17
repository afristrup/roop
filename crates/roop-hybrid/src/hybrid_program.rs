use crate::{Layout, Op};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridProgram {
    pub layout: Layout,
    pub ops: Vec<Op>,
}
