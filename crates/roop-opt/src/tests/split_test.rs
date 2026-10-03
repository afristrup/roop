use roop_syntax::{Block, FnDef, Item, Program};

/// The name of stage `index` of a split test.
pub fn stage_name(test: &str, index: usize) -> String {
    format!("{test}__s{index}")
}

/// The program with one more function for each top-level statement of the
/// named test, each over the same fixtures, so a driver can run the test one
/// statement at a time and look at the state in between. Returns how many
/// statements the test has.
pub fn split_test(program: &Program, test: &str) -> Option<(Program, usize)> {
    let def = program.items.iter().find_map(|item| match item {
        Item::Fn(f) if f.test && f.name == test => Some(f),
        _ => None,
    })?;
    let mut out = program.clone();
    for (i, stmt) in def.body.stmts.iter().enumerate() {
        out.items.push(Item::Fn(FnDef {
            name: stage_name(test, i),
            generics: Vec::new(),
            params: def.params.clone(),
            body: Block {
                stmts: vec![stmt.clone()],
                span: def.body.span,
            },
            irreversible: false,
            public: false,
            test: false,
        }));
    }
    Some((out, def.body.stmts.len()))
}
