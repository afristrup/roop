use crate::OnName;
use roop_syntax::Pattern;

pub fn visit_pattern(pattern: &mut Pattern, on: OnName) {
    if let Pattern::Variant(name, _) = pattern {
        on(name);
    }
}
