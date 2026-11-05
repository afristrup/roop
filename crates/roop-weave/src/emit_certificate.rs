use crate::Model;

/// A Lean certificate for the additive coupling structure of a generated model.
/// The opaque updates must still be related to the emitted Roop layer calls.
pub fn emit_certificate(model: &Model) -> String {
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
    format!(
        "import RoopLean.Certificate\n\nnamespace WeaveCertificate\n\nopen RoopLean\nopen RoopLean.Rev\n\n{updates}\ndef {}_layers : List (CouplingLayer {} 64) := [\n{layers}]\n\ntheorem {}_undo_run (state : {state}) :\n    (couplingNetwork {}_layers).undo ((couplingNetwork {}_layers).exec state) = state :=\n  couplingNetwork_undo_run {}_layers state\n\ntheorem {}_run_undo (state : {state}) :\n    (couplingNetwork {}_layers).exec ((couplingNetwork {}_layers).undo state) = state :=\n  couplingNetwork_run_undo {}_layers state\n\nend WeaveCertificate\n",
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
