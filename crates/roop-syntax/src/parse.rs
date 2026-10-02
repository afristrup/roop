use crate::parser::program;
use crate::{ParseError, Program, Span, Token};
use chumsky::{
    Parser,
    input::{Input, Stream},
};
use logos::Logos;

pub fn parse(src: &str) -> Result<Program, ParseError> {
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
