use crate::{Behaviour, ChoiceKind, Err, Token, TokenInput, comma_list, ident};
use chumsky::prelude::*;

/// `end`, `[checkpoint] offer { a: b, ... }`, `[checkpoint] select { ... }`,
/// `rec x { b }` or a variable. The words are contextual, so they remain
/// usable as names elsewhere.
pub fn behaviour<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Behaviour, Err<'a>> + Clone {
    recursive(|behaviour| {
        let word = |text: &'static str| {
            select! { Token::Ident(s) if s == text => () }
        };
        let end = word("end").to(Behaviour::End);
        let kind = word("offer")
            .to(ChoiceKind::Offer)
            .or(word("select").to(ChoiceKind::Select));
        let branch = ident()
            .then_ignore(just(Token::Colon))
            .then(behaviour.clone());
        let choice = word("checkpoint")
            .or_not()
            .then(kind)
            .then(comma_list(branch).delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map(|((checkpoint, kind), branches)| Behaviour::Choice {
                kind,
                checkpoint: checkpoint.is_some(),
                branches,
            });
        let rec = word("rec")
            .ignore_then(ident())
            .then(behaviour.delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map(|(name, body)| Behaviour::Rec(name, Box::new(body)));
        end.or(choice).or(rec).or(ident().map(Behaviour::Var))
    })
}
