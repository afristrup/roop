use crate::{Activation, Layer, quantize};

/// The call that runs a layer, or its backward step when `back` is set. `into_q`
/// says which half a perceptron adds into.
pub fn layer_call(layer: &Layer, into_q: bool, width: usize, step: f64, back: bool) -> String {
    let kind = layer.activation().map_or(0, Activation::kind);
    let (h, m) = (quantize(step), layer.hidden());
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
        (Layer::Attention { seq, wq, wk, wv }, false) => {
            let (y, x) = if into_q { ("q", "p") } else { ("p", "q") };
            format!(
                "call attn<{seq}, {m}, {width}>({y}, {}, {}, {}, {x});",
                wq.name, wk.name, wv.name
            )
        }
        (Layer::Attention { seq, wq, wk, wv }, true) => {
            let (y, ay, ax, x) = if into_q {
                ("q", "aq", "ap", "p")
            } else {
                ("p", "ap", "aq", "q")
            };
            format!(
                "call attn_back<{seq}, {m}, {width}>({y}, {ay}, {ax}, g{0}, g{1}, g{2}, {0}, {1}, {2}, {x});",
                wq.name, wk.name, wv.name
            )
        }
        (
            Layer::Conv {
                channels,
                kernel,
                w,
                b,
            },
            back,
        ) => {
            let length = width / channels;
            let ck = channels * kernel;
            let generics = format!("{channels}, {kernel}, {length}, {width}, {ck}");
            let (y, ay, ax, x) = if into_q {
                ("q", "aq", "ap", "p")
            } else {
                ("p", "ap", "aq", "q")
            };
            match back {
                false => format!("call conv<{generics}>({y}, {}, {}, {x});", w.name, b.name),
                true => format!(
                    "call conv_back<{generics}>({y}, {ay}, {ax}, g{0}, g{1}, {0}, {1}, {x});",
                    w.name, b.name
                ),
            }
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
