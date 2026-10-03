use crate::{Activation, Layer, batch_suffix, quantize};

/// The call that runs a layer, or its backward step when `back` is set. `into_q`
/// says which half a perceptron adds into, and `batched` that the state holds `B`
/// samples, so that the call is the weave function for a batch.
pub fn layer_call(
    layer: &Layer,
    into_q: bool,
    width: usize,
    step: f64,
    back: bool,
    batched: bool,
) -> String {
    let kind = layer.activation().map_or(0, Activation::kind);
    let (h, m) = (quantize(step), layer.hidden());
    let (s, b) = (batch_suffix(batched), if batched { ", B" } else { "" });
    match (layer, back) {
        (Layer::Leapfrog { w, b: bias, .. }, false) => {
            format!(
                "call layer{s}<{width}, {m}{b}>(q, p, {}, {}, {h}, {kind});",
                w.name, bias.name
            )
        }
        (Layer::Leapfrog { w, b: bias, .. }, true) => format!(
            "call layer_back{s}<{width}, {m}{b}>(q, p, aq, ap, g{0}, g{1}, {0}, {1}, {h}, {kind});",
            w.name, bias.name
        ),
        (
            Layer::Mlp {
                w1,
                b1,
                w2,
                b2,
                norm: Some(norm),
                ..
            },
            false,
        ) => {
            let (y, x) = if into_q { ("q", "p") } else { ("p", "q") };
            format!(
                "call mlp_norm{s}<{width}, {m}{b}>({y}, {}, {}, {}, {}, {}, {x}, {kind}, {});",
                w1.name,
                b1.name,
                norm.gain.name,
                w2.name,
                b2.name,
                quantize(norm.eps)
            )
        }
        (
            Layer::Mlp {
                w1,
                b1,
                w2,
                b2,
                norm: Some(norm),
                ..
            },
            true,
        ) => {
            let (y, ay, ax, x) = if into_q {
                ("q", "aq", "ap", "p")
            } else {
                ("p", "ap", "aq", "q")
            };
            let (g, e) = (&norm.gain.name, quantize(norm.eps));
            format!(
                "call mlp_norm_back{s}<{width}, {m}{b}>({y}, {ay}, {ax}, g{0}, g{1}, g{g2}, g{2}, g{3}, {0}, {1}, {g}, {2}, {3}, {x}, {kind}, {e});",
                w1.name,
                b1.name,
                w2.name,
                b2.name,
                g2 = g
            )
        }
        (Layer::Mlp { w1, b1, w2, b2, .. }, false) => {
            let (y, x) = if into_q { ("q", "p") } else { ("p", "q") };
            format!(
                "call mlp{s}<{width}, {m}{b}>({y}, {}, {}, {}, {}, {x}, {kind});",
                w1.name, b1.name, w2.name, b2.name
            )
        }
        (Layer::Attention { seq, wq, wk, wv }, false) => {
            let (y, x) = if into_q { ("q", "p") } else { ("p", "q") };
            format!(
                "call attn{s}<{seq}, {m}, {width}{b}>({y}, {}, {}, {}, {x});",
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
                "call attn_back{s}<{seq}, {m}, {width}{b}>({y}, {ay}, {ax}, g{0}, g{1}, g{2}, {0}, {1}, {2}, {x});",
                wq.name, wk.name, wv.name
            )
        }
        (
            Layer::Conv {
                channels,
                kernel,
                w,
                b: bias,
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
                false => format!(
                    "call conv{s}<{generics}{b}>({y}, {}, {}, {x});",
                    w.name, bias.name
                ),
                true => format!(
                    "call conv_back{s}<{generics}{b}>({y}, {ay}, {ax}, g{0}, g{1}, {0}, {1}, {x});",
                    w.name, bias.name
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
                "call mlp_back{s}<{width}, {m}{b}>({y}, {ay}, {ax}, g{0}, g{1}, g{2}, g{3}, {0}, {1}, {2}, {3}, {x}, {kind});",
                w1.name, b1.name, w2.name, b2.name
            )
        }
    }
}
