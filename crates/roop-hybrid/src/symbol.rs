use crate::Token;

/// The operator spelled by `c`, and `next` when it takes two characters.
pub fn symbol(c: char, next: Option<char>) -> Option<(Token, usize)> {
    let two = match (c, next) {
        (':', Some('=')) => Some(Token::Assign),
        ('<', Some('=')) => Some(Token::Le),
        ('&', Some('&')) => Some(Token::And),
        ('|', Some('|')) => Some(Token::Or),
        _ => None,
    };
    if let Some(token) = two {
        return Some((token, 2));
    }
    let one = match c {
        ';' => Token::Semi,
        ',' => Token::Comma,
        '(' => Token::LParen,
        ')' => Token::RParen,
        '+' => Token::Plus,
        '*' => Token::Star,
        '^' => Token::Caret,
        '=' => Token::Eq,
        '≤' => Token::Le,
        '∧' => Token::And,
        '∨' => Token::Or,
        '¬' | '!' => Token::Not,
        _ => return None,
    };
    Some((one, 1))
}
