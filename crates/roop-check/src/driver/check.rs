use crate::{
    CheckError, Scope, check_block, check_concurrency, check_resolution, check_session,
    check_struct, enum_table, fn_mutability, irreversible_fns, non_atomic_fns, program_blocks,
    world_fns,
};
use roop_syntax::{Item, Program};

pub fn check(program: &Program) -> Result<(), CheckError> {
    let enums = enum_table(program)?;
    for block in program_blocks(program) {
        check_resolution(&enums, block)?;
    }
    check_concurrency(program)?;
    let irreversible = irreversible_fns(program);
    let non_atomic = non_atomic_fns(program);
    let world = world_fns(program);
    let mutability = fn_mutability(program);
    let reversible = Scope {
        irrev: false,
        logged: false,
        irreversible_fns: &irreversible,
        non_atomic_fns: &non_atomic,
        world_fns: &world,
        mutability: &mutability,
        facts: None,
    };
    for item in &program.items {
        match item {
            Item::Fn(f) => {
                let scope = Scope {
                    irrev: f.irreversible,
                    ..reversible
                };
                check_block(&f.body, scope)?
            }
            Item::Struct(def) => check_struct(def, reversible)?,
            Item::Session(def) => check_session(def)?,
            Item::Mod(_) | Item::Use(_) | Item::Enum(_) => {}
        }
    }
    Ok(())
}
