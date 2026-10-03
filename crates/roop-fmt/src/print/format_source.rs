use crate::{Ctx, FmtError, print_program, render, same_program};
use roop_config::FormatConfig;
use roop_syntax::{comments, parse_spanned};

/// Lays out a program: the same program and the same comments, in lines of at
/// most `config.max_width` columns where the code allows it.
pub fn format_source(src: &str, config: &FormatConfig) -> Result<String, FmtError> {
    let items = parse_spanned(src).map_err(FmtError::Parse)?;
    let ctx = Ctx::new(src, comments(src));
    let doc = print_program(&ctx, &items);
    if let Some(line) = ctx.stray_line() {
        return Err(FmtError::Comment(line));
    }
    let out = render(&doc, config.max_width, config.indent);
    if !same_program(src, &out) {
        return Err(FmtError::Changed);
    }
    Ok(out)
}
