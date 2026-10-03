use crate::{Layer, Model, Tensor};

/// z = W x + b
fn affine(w: &Tensor, b: &Tensor, x: &[f64]) -> Vec<f64> {
    (0..w.dims[0])
        .map(|j| b.data[j] + (0..w.dims[1]).map(|i| w.at(j, i) * x[i]).sum::<f64>())
        .collect()
}

fn leapfrog(layer: &Layer, h: f64, q: &mut [f64], p: &mut [f64]) {
    let Layer::Leapfrog { act, w, b } = layer else {
        unreachable!("a leapfrog layer")
    };
    let kick = |q: &[f64], p: &mut [f64]| {
        let s: Vec<f64> = affine(w, b, q).into_iter().map(|z| act.eval(z)).collect();
        for (i, p) in p.iter_mut().enumerate() {
            *p += -h / 2.0 * (0..s.len()).map(|j| w.at(j, i) * s[j]).sum::<f64>();
        }
    };
    kick(q, p);
    q.iter_mut().zip(p.iter()).for_each(|(q, p)| *q += h * p);
    kick(q, p);
}

fn mlp(layer: &Layer, y: &mut [f64], x: &[f64]) {
    let Layer::Mlp {
        act,
        w1,
        b1,
        w2,
        b2,
    } = layer
    else {
        unreachable!("an mlp layer")
    };
    let a: Vec<f64> = affine(w1, b1, x).into_iter().map(|z| act.eval(z)).collect();
    y.iter_mut()
        .zip(affine(w2, b2, &a))
        .for_each(|(y, f)| *y += f);
}

/// The output of the network, the first `outputs` numbers of q after every
/// layer, in doubles, for an input in q and zero in p.
pub fn forward(model: &Model, input: &[f64]) -> Vec<f64> {
    let mut q = input.to_vec();
    let mut p = vec![0.0; q.len()];
    for (layer, into_q) in model.layers.iter().zip(model.adds_into_q()) {
        match (layer, into_q) {
            (Layer::Leapfrog { .. }, _) => leapfrog(layer, model.step, &mut q, &mut p),
            (Layer::Mlp { .. }, true) => mlp(layer, &mut q, &p.clone()),
            (Layer::Mlp { .. }, false) => mlp(layer, &mut p, &q.clone()),
        }
    }
    q.truncate(model.outputs);
    q
}
