use crate::{Pattern, Visitor};

pub fn walk_pattern(v: &mut dyn Visitor, pattern: &mut Pattern) {
    if let Pattern::Variant(name, _) = pattern {
        v.name(name);
    }
}
