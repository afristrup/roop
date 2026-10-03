use roop_syntax::ParseError;
use std::fmt;

#[derive(Debug)]
pub enum FmtError {
    Parse(ParseError),
    /// A comment sits where the formatter cannot keep it in place.
    Comment(usize),
    /// The formatted program does not parse to the same program.
    Changed,
}

impl fmt::Display for FmtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Comment(line) => write!(
                f,
                "line {line}: a comment inside a statement header or a declaration cannot be kept in place"
            ),
            Self::Changed => write!(f, "formatting would change the program, this is a bug"),
        }
    }
}

impl std::error::Error for FmtError {}
