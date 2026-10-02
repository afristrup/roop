use logos::Logos;
use std::fmt;

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip(r"//[^\n]*", allow_greedy = true))]
pub enum Token<'a> {
    #[token("mod")]
    Mod,
    #[token("use")]
    Use,
    #[token("pub")]
    Pub,
    #[token("as")]
    As,
    #[token("irrev")]
    Irrev,
    #[token("enum")]
    Enum,
    #[token("struct")]
    Struct,
    #[token("fn")]
    Fn,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("fi")]
    Fi,
    #[token("from")]
    From,
    #[token("loop")]
    Loop,
    #[token("until")]
    Until,
    #[token("ancilla")]
    Ancilla,
    #[token("call")]
    Call,
    #[token("uncall")]
    Uncall,
    #[token("mut")]
    Mut,
    #[token("borrow")]
    Borrow,
    #[token("chan")]
    Chan,
    #[token("send")]
    Send,
    #[token("recv")]
    Recv,
    #[token("push")]
    Push,
    #[token("pop")]
    Pop,
    #[token("logged")]
    Logged,
    #[token("empty")]
    Empty,
    #[token("try")]
    Try,
    #[token("catch_rollback")]
    CatchRollback,
    #[token("build")]
    Build,
    #[token("unbuild")]
    Unbuild,
    #[token("match")]
    Match,
    #[token("assert")]
    Assert,
    #[token("_")]
    Underscore,
    #[token("true")]
    True,
    #[token("false")]
    False,

    #[regex(r"[A-Za-z][A-Za-z0-9_]*|_[A-Za-z0-9_]+")]
    Ident(&'a str),
    #[regex(r"[0-9]+")]
    Int(&'a str),
    #[regex(r"[0-9]+\.[0-9]+")]
    Float(&'a str),

    #[token("#")]
    Hash,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("->")]
    Arrow,
    #[token("<-")]
    LArrow,
    #[token("=>")]
    FatArrow,
    #[token("=")]
    Assign,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("%=")]
    PercentEq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("^=")]
    CaretEq,
    #[token("<=>")]
    Swap,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("&")]
    Amp,
    #[token("&&")]
    AmpAmp,
    #[token("||")]
    PipePipe,
    #[token("!")]
    Bang,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ident(s) | Self::Int(s) | Self::Float(s) => write!(f, "{s}"),
            other => write!(f, "{other:?}"),
        }
    }
}
