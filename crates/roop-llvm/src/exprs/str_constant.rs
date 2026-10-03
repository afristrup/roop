use crate::FnGen;

/// A private constant holding `bytes`, as the address of its first byte.
pub fn str_constant(g: &mut FnGen, bytes: &[u8]) -> String {
    let id = g.fresh("s");
    let label = format!("@{}.str{id}", g.symbol);
    let text: String = bytes.iter().map(|b| format!("\\{b:02X}")).collect();
    g.globals.push(format!(
        "{label} = private unnamed_addr constant [{} x i8] c\"{text}\"\n",
        bytes.len()
    ));
    label
}
