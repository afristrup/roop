use crate::Error;

#[derive(Debug, PartialEq)]
pub struct Syrec {
    pub file: String,
    pub module: String,
}

pub fn parse_syrec(file: &str, source: &str) -> Result<Syrec, Error> {
    let invalid = |message| Error::Invalid {
        file: file.into(),
        message,
    };
    let header = source
        .lines()
        .map(str::trim_start)
        .find_map(|line| line.strip_prefix("module "))
        .ok_or_else(|| invalid("missing module header".into()))?;
    let (name, _) = header
        .split_once("(")
        .ok_or_else(|| invalid("missing parameter list".into()))?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == 95)
    {
        return Err(invalid("invalid module name".into()));
    }
    if !source.contains(")") {
        return Err(invalid("unterminated parameter list".into()));
    }
    Ok(Syrec {
        file: file.into(),
        module: name.into(),
    })
}
