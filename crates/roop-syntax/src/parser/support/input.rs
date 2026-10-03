use crate::{Span, Token};
use chumsky::input::ValueInput;

pub trait TokenInput<'a>: ValueInput<'a, Token = Token<'a>, Span = Span> {}

impl<'a, I> TokenInput<'a> for I where I: ValueInput<'a, Token = Token<'a>, Span = Span> {}
