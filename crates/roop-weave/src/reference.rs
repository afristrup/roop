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

/// y += (Q K^T) V for x read as `seq` rows, with Q = X Wq^T and so on.
fn attention(layer: &Layer, y: &mut [f64], x: &[f64]) {
    let Layer::Attention { seq, wq, wk, wv } = layer else {
        unreachable!("an attention layer")
    };
    let d = wq.dims[0];
    let project = |w: &Tensor| -> Vec<Vec<f64>> {
        (0..*seq)
            .map(|s| {
                (0..d)
                    .map(|e| (0..d).map(|i| x[s * d + i] * w.at(e, i)).sum())
                    .collect()
            })
            .collect()
    };
    let (q, k, v) = (project(wq), project(wk), project(wv));
    for s in 0..*seq {
        for t in 0..*seq {
            let score: f64 = (0..d).map(|e| q[s][e] * k[t][e]).sum();
            (0..d).for_each(|e| y[s * d + e] += score * v[t][e]);
        }
    }
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
            (Layer::Attention { .. }, true) => attention(layer, &mut q, &p.clone()),
            (Layer::Attention { .. }, false) => attention(layer, &mut p, &q.clone()),
        }
    }
    q.truncate(model.outputs);
    q
}
