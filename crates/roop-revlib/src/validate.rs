use crate::{Error, Pla};

pub fn validate(pla: &Pla) -> Result<(), Error> {
    for (input, output) in &pla.rows {
        if input.len() != pla.inputs || output.len() != pla.outputs {
            return Err(Error::Invalid {
                file: pla.file.clone(),
                message: "row width disagrees with header".into(),
            });
        }
        if !input
            .bytes()
            .chain(output.bytes())
            .all(|bit| matches!(bit, b'0' | b'1' | b'-' | b'~'))
        {
            return Err(Error::Invalid {
                file: pla.file.clone(),
                message: "row contains a non-PLA bit".into(),
            });
        }
    }
    Ok(())
}
