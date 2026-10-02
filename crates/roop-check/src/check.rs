use super::{CheckError, check_block};
use roop_syntax::{Item, Program};

pub fn check(program: &Program) -> Result<(), CheckError> {
    for item in &program.items {
        if let Item::Fn(f) = item {
            check_block(&f.body)?;
        }
    }
    Ok(())
}
