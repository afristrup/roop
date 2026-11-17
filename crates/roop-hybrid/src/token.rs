use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    Num(i64),
    Assign,
    Semi,
    Comma,
    LParen,
    RParen,
    Plus,
    Star,
    Caret,
    Le,
    Eq,
    And,
    Or,
    Not,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ident(name) => write!(f, "`{name}`"),
            Self::Num(n) => write!(f, "`{n}`"),
            Self::Assign => write!(f, "`:=`"),
            Self::Semi => write!(f, "`;`"),
            Self::Comma => write!(f, "`,`"),
            Self::LParen => write!(f, "`(`"),
            Self::RParen => write!(f, "`)`"),
            Self::Plus => write!(f, "`+`"),
            Self::Star => write!(f, "`*`"),
            Self::Caret => write!(f, "`^`"),
            Self::Le => write!(f, "`<=`"),
            Self::Eq => write!(f, "`=`"),
            Self::And => write!(f, "`&&`"),
            Self::Or => write!(f, "`||`"),
            Self::Not => write!(f, "`!`"),
            Self::Eof => write!(f, "the end of the file"),
        }
    }
}
