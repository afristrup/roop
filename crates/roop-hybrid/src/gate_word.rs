use crate::GateName;

/// The gate a word names; `R3` is R_3. `R` alone takes k as its last argument.
pub fn gate_word(word: &str) -> Option<GateName> {
    Some(match word {
        "X" => GateName::X,
        "Z" => GateName::Z,
        "H" => GateName::H,
        "S" => GateName::S,
        "T" => GateName::T,
        "CNOT" | "CX" => GateName::Cnot,
        _ => GateName::R(word.strip_prefix('R')?.parse().ok()?),
    })
}
