use crate::{EinsumError, Spec};

/// Reads `"ij,jk->ik"`. Without `->` the output is the labels that appear
/// once, in alphabetical order, as in NumPy.
pub fn parse_spec(name: &str, text: &str) -> Result<Spec, EinsumError> {
    let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if let Some(bad) = text
        .chars()
        .find(|c| !(c.is_ascii_alphabetic() || matches!(c, ',' | '-' | '>')))
    {
        return Err(EinsumError::BadCharacter {
            name: name.into(),
            found: bad,
        });
    }
    let (left, right) = match text.split_once("->") {
        Some((left, right)) => (left, Some(right)),
        None => (text.as_str(), None),
    };
    if left.is_empty() {
        return Err(EinsumError::Empty(name.into()));
    }
    let inputs: Vec<Vec<char>> = left.split(',').map(|part| part.chars().collect()).collect();
    if let Some(bad) = text
        .replace("->", "")
        .chars()
        .find(|c| !c.is_ascii_alphabetic() && *c != ',')
    {
        return Err(EinsumError::BadCharacter {
            name: name.into(),
            found: bad,
        });
    }
    let output: Vec<char> = match right {
        Some(right) => right.chars().collect(),
        None => {
            let mut once: Vec<char> = inputs
                .iter()
                .flatten()
                .copied()
                .filter(|l| inputs.iter().flatten().filter(|m| *m == l).count() == 1)
                .collect();
            once.sort();
            once
        }
    };
    for (k, label) in output.iter().enumerate() {
        if !inputs.iter().flatten().any(|l| l == label) {
            return Err(EinsumError::MissingLabel {
                name: name.into(),
                label: *label,
            });
        }
        if output[..k].contains(label) {
            return Err(EinsumError::RepeatedOutput {
                name: name.into(),
                label: *label,
            });
        }
    }
    Ok(Spec { inputs, output })
}
