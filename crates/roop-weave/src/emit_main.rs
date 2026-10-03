use crate::{Model, names};

/// A `main` that runs the model: it holds the weights, reads the input as `width`
/// whole numbers on the 1/4096 grid from the command line, and prints each output
/// on a line, on the same grid.
pub fn emit_main(model: &Model) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let tensors = model.tensors();
    let declared: String = tensors
        .iter()
        .map(|t| format!("    auto ancilla {}: {} = 0;\n", t.name, t.roop_type()))
        .collect();
    let weights = names(&tensors).join(", ");
    format!(
        "use std::io::println_int;
use std::process::arg;
use std::process::arg_count;
use std::text::parse_int;

fn main() {{
{declared}    auto ancilla q: [i64; {n}] = 0;
    auto ancilla p: [i64; {n}] = 0;
    ancilla argc: i64 = 0;
    ancilla o: i64 = 0;
    call {name}_load({weights});
    call arg_count(argc);
    expect argc == {n} + 1;
    ancilla i: i64 = 0;
    from i == 0 {{
        auto ancilla word: [u8; 64] = 0;
        auto ancilla len: i64 = 0;
        auto ancilla value: i64 = 0;
        auto ancilla ok: i64 = 0;
        call arg(i + 1, word, len);
        call parse_int(word, len, value, ok);
        q[i] += value;
    }} loop {{ i += 1; }} until i == {n} - 1;
    i -= {n} - 1;
    call {name}_forward(q, p, {weights});
    from o == 0 {{
        call println_int(q[o]);
    }} loop {{ o += 1; }} until o == {k} - 1;
    o -= {k} - 1;
    uncall arg_count(argc);
}}
"
    )
}
