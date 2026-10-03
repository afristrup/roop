use roop_syntax::{Item, Program};

/// The program without its `test` items, as `roop build` and `roop lean` want it.
pub fn strip_tests(program: &Program) -> Program {
    let keep = |item: &&Item| !matches!(item, Item::Fn(f) if f.test);
    Program {
        items: program.items.iter().filter(keep).cloned().collect(),
    }
}
