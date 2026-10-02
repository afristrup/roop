use crate::Program;
use crate::{Err, TokenInput, item};
use chumsky::prelude::*;

pub fn program<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Program, Err<'a>> {
    item()
        .repeated()
        .collect()
        .then_ignore(end())
        .map(|items| Program { items })
}
