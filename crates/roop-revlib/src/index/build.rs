use crate::{Error, Index, Node, Pla, validate};

pub fn index(pla: &Pla) -> Result<Index, Error> {
    validate(pla)?;
    let mut nodes = vec![Node::default()];
    for (cube, output) in &pla.rows {
        let mut cursor = 0;
        for bit in cube.bytes() {
            let existing = match bit {
                b'0' => nodes[cursor].zero,
                b'1' => nodes[cursor].one,
                b'-' => nodes[cursor].any,
                _ => unreachable!(),
            };
            cursor = match existing {
                Some(next) => next,
                None => {
                    let next = nodes.len();
                    nodes.push(Node::default());
                    match bit {
                        b'0' => nodes[cursor].zero = Some(next),
                        b'1' => nodes[cursor].one = Some(next),
                        b'-' => nodes[cursor].any = Some(next),
                        _ => unreachable!(),
                    }
                    next
                }
            };
        }
        nodes[cursor].outputs.push(output.clone());
    }
    Ok(Index {
        file: pla.file.clone(),
        inputs: pla.inputs,
        node_count: nodes.len(),
        nodes,
    })
}
