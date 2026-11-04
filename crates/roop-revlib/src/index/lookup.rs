use crate::{Error, Index};

pub fn lookup<'a>(index: &'a Index, input: &str) -> Result<Vec<&'a str>, Error> {
    if input.len() != index.inputs || !input.bytes().all(|bit| matches!(bit, b'0' | b'1')) {
        return Err(Error::Invalid {
            file: index.file.clone(),
            message: "input is not a concrete PLA bit vector".into(),
        });
    }
    let mut cursors = vec![0];
    for bit in input.bytes() {
        let mut next = Vec::new();
        for cursor in cursors {
            let node = &index.nodes[cursor];
            let specific = if bit == b'0' { node.zero } else { node.one };
            next.extend(specific.into_iter().chain(node.any));
        }
        cursors = next;
    }
    Ok(cursors
        .into_iter()
        .flat_map(|cursor| index.nodes[cursor].outputs.iter().map(String::as_str))
        .collect())
}
