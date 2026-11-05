use crate::Model;

/// A Lean certificate for the additive coupling structure of a generated model.
/// The source fingerprint identifies its generated Roop source for review.
pub fn emit_certificate(model: &Model, source: &str) -> String {
    let directions = model.certificate_directions();
    let updates: String = directions
        .iter()
        .enumerate()
        .map(|(i, _)| {
            format!(
                "opaque {}_update_{i} : Tensor {} 64 -> Tensor {} 64\n",
                model.name, model.width, model.width
            )
        })
        .collect();
    let layers: String = directions
        .iter()
        .enumerate()
        .map(|(i, into_first)| {
            format!(
                "  {{ intoFirst := {into_first}, update := {}_update_{i} }},\n",
                model.name
            )
        })
        .collect();
    let state = format!("Tensor {} 64 × Tensor {} 64", model.width, model.width);
    let fingerprint = source_fingerprint(source);
    format!(
        "-- Roop source FNV-1a fingerprint: {fingerprint:016x}\nimport RoopLean.Certificate\n\nnamespace WeaveCertificate\n\nopen RoopLean\nopen RoopLean.Rev\n\n{updates}\ndef {}_layers : List (CouplingLayer {} 64) := [\n{layers}]\n\ntheorem {}_undo_run (state : {state}) :\n    (couplingNetwork {}_layers).undo ((couplingNetwork {}_layers).exec state) = state :=\n  couplingNetwork_undo_run {}_layers state\n\ntheorem {}_run_undo (state : {state}) :\n    (couplingNetwork {}_layers).exec ((couplingNetwork {}_layers).undo state) = state :=\n  couplingNetwork_run_undo {}_layers state\n\nend WeaveCertificate\n",
        model.name,
        model.width,
        model.name,
        model.name,
        model.name,
        model.name,
        model.name,
        model.name,
        model.name,
        model.name,
    )
}

pub fn source_fingerprint(source: &str) -> u64 {
    source.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
