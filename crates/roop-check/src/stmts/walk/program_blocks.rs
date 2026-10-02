use roop_syntax::{Block, Item, Program};

/// Every function body and struct constructor body.
pub fn program_blocks(program: &Program) -> Vec<&Block> {
    let mut blocks = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) => blocks.push(&f.body),
            Item::Struct(def) => blocks.extend(
                [&def.build, &def.unbuild]
                    .into_iter()
                    .flatten()
                    .map(|b| &b.body),
            ),
            Item::Mod(_) | Item::Use(_) | Item::Enum(_) | Item::Session(_) => {}
        }
    }
    blocks
}
