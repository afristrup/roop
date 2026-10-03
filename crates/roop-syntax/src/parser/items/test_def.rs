use crate::{Block, FnDef, Item, Param, Token, Type};
use crate::{Err, TokenInput, comma_list, entries, fold_entries, ident, param, stmt};
use chumsky::prelude::*;

/// `test name { x: i64, y: [i64; 4]; statements }`. The fixtures start at zero
/// and are the parameters of a function the runner calls.
pub fn test_def<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Item, Err<'a>> + Clone {
    let fixtures = comma_list(param()).then_ignore(just(Token::Semi));
    let body = fixtures
        .or_not()
        .then(entries(stmt()))
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .map_with(|(fixtures, entries), e| {
            let span: crate::Span = e.span();
            let stmts = fold_entries(entries, span.end);
            (fixtures.unwrap_or_default(), Block { stmts, span })
        });
    select! { Token::Ident("test") => () }
        .ignore_then(ident())
        .then(body)
        .map(|(name, (fixtures, body))| {
            let params = fixtures.into_iter().map(by_reference).collect();
            Item::Fn(FnDef {
                name,
                generics: Vec::new(),
                params,
                body,
                irreversible: false,
                public: false,
                test: true,
                bennett: None,
                external: false,
                world: false,
                einsum: None,
            })
        })
}

fn by_reference(fixture: Param) -> Param {
    Param {
        name: fixture.name,
        ty: Type::Ref {
            mutable: true,
            inner: Box::new(fixture.ty),
        },
    }
}
