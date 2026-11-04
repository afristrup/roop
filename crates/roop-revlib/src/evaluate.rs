use crate::{Error, Pla, validate};

pub fn evaluate<'a>(pla: &'a Pla, input: &str) -> Result<Vec<&'a str>, Error> {
    validate(pla)?;
    if input.len() != pla.inputs || !input.bytes().all(|bit| matches!(bit, b'0' | b'1')) {
        return Err(Error::Invalid {
            file: pla.file.clone(),
            message: "input is not a concrete PLA bit vector".into(),
        });
    }
    Ok(pla
        .rows
        .iter()
        .filter(|(cube, _)| {
            cube.bytes()
                .zip(input.bytes())
                .all(|(expected, actual)| expected == b'-' || expected == actual)
        })
        .map(|(_, output)| output.as_str())
        .collect())
}
