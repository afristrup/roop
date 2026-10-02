use crate::{CheckError, program_blocks, walk_channels};
use roop_syntax::Program;

pub fn check_concurrency(program: &Program) -> Result<(), CheckError> {
    program_blocks(program)
        .into_iter()
        .try_for_each(|block| walk_channels(block, &mut Vec::new(), false))
}
