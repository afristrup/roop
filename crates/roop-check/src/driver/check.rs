use crate::{CheckError, check_block, check_resolution, check_struct, enum_table, program_blocks};
use roop_syntax::{Item, Program};

pub fn check(program: &Program) -> Result<(), CheckError> {
    let enums = enum_table(program)?;
    for block in program_blocks(program) {
        check_resolution(&enums, block)?;
    }
    for item in &program.items {
        match item {
            Item::Fn(f) => check_block(&f.body)?,
            Item::Struct(def) => check_struct(def)?,
            Item::Mod(_) | Item::Use(_) | Item::Enum(_) => {}
        }
    }
    Ok(())
}
