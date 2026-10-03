use crate::{Layer, quantize};

/// The call that runs a layer, or its backward step when `back` is set. `into_q`
/// says which half a perceptron adds into.
pub fn layer_call(layer: &Layer, into_q: bool, width: usize, step: f64, back: bool) -> String {
    let (h, kind, m) = (quantize(step), layer.activation().kind(), layer.hidden());
    match (layer, back) {
        (Layer::Leapfrog { w, b, .. }, false) => {
            format!(
                "call layer<{width}, {m}>(q, p, {}, {}, {h}, {kind});",
                w.name, b.name
            )
        }
        (Layer::Leapfrog { w, b, .. }, true) => format!(
            "call layer_back<{width}, {m}>(q, p, aq, ap, g{0}, g{1}, {0}, {1}, {h}, {kind});",
            w.name, b.name
        ),
        (Layer::Mlp { w1, b1, w2, b2, .. }, false) => {
            let (y, x) = if into_q { ("q", "p") } else { ("p", "q") };
            format!(
                "call mlp<{width}, {m}>({y}, {}, {}, {}, {}, {x}, {kind});",
                w1.name, b1.name, w2.name, b2.name
            )
        }
        (Layer::Mlp { w1, b1, w2, b2, .. }, true) => {
            let (y, ay, ax, x) = if into_q {
                ("q", "aq", "ap", "p")
            } else {
                ("p", "ap", "aq", "q")
            };
            format!(
                "call mlp_back<{width}, {m}>({y}, {ay}, {ax}, g{0}, g{1}, g{2}, g{3}, {0}, {1}, {2}, {3}, {x}, {kind});",
                w1.name, b1.name, w2.name, b2.name
            )
        }
    }
}
