use crate::{AutoError, keep_in_region, keep_stmt};
use roop_syntax::Block;

/// Lets go of the ancilla `name` where its block ends, like the end of a
/// lifetime, and at the end of each run of its region if it has one.
pub fn release(name: &str, region: Option<&str>, body: &mut Block) -> Result<(), AutoError> {
    if let Some(region) = region
        && !keep_in_region(name, region, body)?
    {
        return Err(AutoError::UnknownRegion {
            name: name.to_string(),
            region: region.to_string(),
        });
    }
    let at = body.span.end;
    body.stmts.push(keep_stmt(name, at));
    Ok(())
}
