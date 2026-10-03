use roop_syntax::Type;

/// The name of the length parameter for a label.
pub fn length_name(label: char) -> String {
    format!("n_{label}")
}

/// `[[T; n_b]; n_a]` for the labels `a`, `b`: the first label is the outermost.
pub fn operand_type(elem: &Type, labels: &[char]) -> Type {
    labels
        .iter()
        .rev()
        .fold(elem.clone(), |inner, label| Type::Param {
            elem: Box::new(inner),
            len: length_name(*label),
            stack: false,
        })
}
