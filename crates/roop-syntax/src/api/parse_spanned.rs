use crate::program;
use crate::{Item, ParseError, Span, Token};
use chumsky::{
    Parser,
    input::{Input, Stream},
};
use logos::Logos;

/// Parses a program and keeps the span of each item.
pub fn parse_spanned(src: &str) -> Result<Vec<(Item, Span)>, ParseError> {
    let tokens = Token::lexer(src)
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(tok) => Ok((tok, Span::from(span))),
            Err(()) => Err(ParseError {
                message: "unrecognized token".into(),
                span: span.into(),
            }),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let eoi = Span::from(src.len()..src.len());
    let stream = Stream::from_iter(tokens).map(eoi, |(t, s)| (t, s));
    program().parse(stream).into_result().map_err(|errs| {
        let first = &errs[0];
        ParseError {
            message: first.to_string(),
            span: *first.span(),
        }
    })
}
