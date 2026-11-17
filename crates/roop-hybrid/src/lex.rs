use crate::{HybridError, Spanned, Token, symbol, word};

pub fn lex(text: &str) -> Result<Vec<Spanned>, HybridError> {
    let chars: Vec<char> = text.chars().collect();
    let end = |from: usize, keep: fn(char) -> bool| {
        (from..chars.len())
            .find(|&j| !keep(chars[j]))
            .unwrap_or(chars.len())
    };
    let (mut i, mut line, mut tokens) = (0, 1, Vec::new());
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        let (token, to) = if c == '\n' {
            line += 1;
            (None, i + 1)
        } else if c.is_whitespace() {
            (None, i + 1)
        } else if c == '/' && next == Some('/') {
            (None, end(i, |c| c != '\n'))
        } else if c.is_ascii_digit() {
            let to = end(i, |c| c.is_ascii_digit());
            let text: String = chars[i..to].iter().collect();
            let n = text
                .parse()
                .map_err(|_| HybridError::Number { line, text })?;
            (Some(Token::Num(n)), to)
        } else if c.is_alphabetic() || c == '_' {
            let to = end(i, |c| c.is_alphanumeric() || c == '_');
            (Some(word(&chars[i..to].iter().collect::<String>())), to)
        } else {
            let (token, width) = symbol(c, next).ok_or(HybridError::Lex { line, found: c })?;
            (Some(token), i + width)
        };
        if let Some(token) = token {
            tokens.push(Spanned { token, line });
        }
        i = to;
    }
    tokens.push(Spanned {
        token: Token::Eof,
        line,
    });
    Ok(tokens)
}
