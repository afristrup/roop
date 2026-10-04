use crate::{Layer, Model};

/// The calls that scale the weights of each residual block that has `keep_contraction`
/// back to its bound, after the optimizer has moved them.
pub fn emit_project(model: &Model) -> String {
    let project = |layer: &Layer| match layer {
        Layer::Residual {
            act,
            w1,
            w2,
            keep: Some(keep),
            ..
        } => {
            let slope = (act.lipschitz() * 4096.0).ceil() as i64;
            let cap = (keep * 4096.0).floor() as i64;
            Some(format!(
                "    call project_contraction<{}, {}>({}, {}, {slope}, {cap});\n",
                w1.dims[1], w1.dims[0], w1.name, w2.name
            ))
        }
        _ => None,
    };
    model.layers.iter().filter_map(project).collect()
}
