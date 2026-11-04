use crate::Error;

#[derive(Debug, PartialEq)]
pub struct Pla {
    pub file: String,
    pub inputs: usize,
    pub outputs: usize,
    pub rows: Vec<(String, String)>,
}

pub fn parse(file: &str, source: &str) -> Result<Pla, Error> {
    let invalid = |message| Error::Invalid {
        file: file.into(),
        message,
    };
    let mut inputs = None;
    let mut outputs = None;
    let mut rows = Vec::new();
    for line in source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if line.starts_with('#') || line == ".e" {
            continue;
        }
        if let Some(value) = line.strip_prefix(".i ") {
            inputs = Some(
                value
                    .parse()
                    .map_err(|_| invalid("invalid input count".into()))?,
            );
            continue;
        }
        if let Some(value) = line.strip_prefix(".o ") {
            outputs = Some(
                value
                    .parse()
                    .map_err(|_| invalid("invalid output count".into()))?,
            );
            continue;
        }
        if line.starts_with('.') {
            continue;
        }
        let mut bits = line.split_whitespace();
        let Some(input) = bits.next() else { continue };
        let output: String = bits.collect();
        if output.is_empty() {
            return Err(invalid("missing output row".into()));
        }
        rows.push((input.into(), output));
    }
    Ok(Pla {
        file: file.into(),
        inputs: inputs.ok_or_else(|| invalid("missing .i header".into()))?,
        outputs: outputs.ok_or_else(|| invalid("missing .o header".into()))?,
        rows,
    })
}
