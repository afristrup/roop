use super::{CheckError, check_block, check_struct};
use roop_syntax::{Item, Program};

pub fn check(program: &Program) -> Result<(), CheckError> {
    for item in &program.items {
        match item {
            Item::Fn(f) => check_block(&f.body)?,
            Item::Struct(def) => check_struct(def)?,
            Item::Mod(_) | Item::Use(_) => {}
        }
    }
    Ok(())
}
