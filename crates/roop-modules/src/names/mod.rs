mod global_name;
mod item_name;
mod visit_block;
mod visit_expr;
mod visit_item;
mod visit_pattern;
mod visit_place;
mod visit_stmt;
mod visit_type;

pub use global_name::*;
pub use item_name::*;
pub use visit_block::*;
pub use visit_expr::*;
pub use visit_item::*;
pub use visit_pattern::*;
pub use visit_place::*;
pub use visit_stmt::*;
pub use visit_type::*;

/// Called on every name that refers to a function, struct or enum.
pub type OnName<'a> = &'a mut dyn FnMut(&mut String);
