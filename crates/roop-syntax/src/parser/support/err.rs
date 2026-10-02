use crate::Token;
use chumsky::{extra, prelude::Rich};

pub type Err<'a> = extra::Err<Rich<'a, Token<'a>>>;
