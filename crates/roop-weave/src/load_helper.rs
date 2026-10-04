/// `fn name(row: &mut [i64; len])` that adds each quantized weight at its index.
pub fn load_helper(name: &str, len: usize, writes: &[(usize, i64)]) -> String {
    let body: String = writes
        .iter()
        .map(|(i, q)| {
            let sign = if *q < 0 { '-' } else { '+' };
            format!("    row[{i}] {sign}= {};\n", q.abs())
        })
        .collect();
    format!("fn {name}(row: &mut [i64; {len}]) {{\n{body}}}\n\n")
}
