use crate::{Err, TokenInput, item};
use crate::{Item, Span};
use chumsky::prelude::*;

pub fn program<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Vec<(Item, Span)>, Err<'a>> {
    item()
        .map_with(|item, e| (item, e.span()))
        .repeated()
        .collect()
        .then_ignore(end())
}
