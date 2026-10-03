/// A proof by the call-by-call script when there is one, with the general
/// script as what to try if that does not go through.
pub fn guarded_proof(chain: Option<String>, general: String) -> String {
    let Some(chain) = chain else {
        return general;
    };
    let indent = |text: &str| -> String {
        text.lines()
            .map(|line| format!("      {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let (chain, general) = (indent(&chain), indent(&general));
    format!("  first\n    | (\n{chain})\n    | (\n{general})\n")
}
