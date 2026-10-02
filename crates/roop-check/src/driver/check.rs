use crate::{
    CheckError, Scope, check_block, check_concurrency, check_resolution, check_struct, enum_table,
    irreversible_fns, program_blocks,
};
use roop_syntax::{Item, Program};

pub fn check(program: &Program) -> Result<(), CheckError> {
    let enums = enum_table(program)?;
    for block in program_blocks(program) {
        check_resolution(&enums, block)?;
    }
    check_concurrency(program)?;
    let irreversible = irreversible_fns(program);
    let reversible = Scope {
        irrev: false,
        irreversible_fns: &irreversible,
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
            Item::Mod(_) | Item::Use(_) | Item::Enum(_) => {}
        }
    }
    Ok(())
}
