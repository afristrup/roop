use crate::Item;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Program {
    pub items: Vec<Item>,
}
