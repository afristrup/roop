// The same network as roop/weave, trained the usual way: the backward pass reads
// activations that the forward pass stored, either all of them or every
// sqrt(L)-th one with the rest recomputed. The arithmetic is the integer
// arithmetic of roop/weave/kernels.roop, term by term, so the gradients can be
// compared with roop's bit for bit.
//
//     weave <stored|checkpoint> N L B
//
// B samples are in flight at once, as a framework would run a batch, and the
// output is the gradients' checksum and the bytes the activations took.
use std::time::Instant;

const Q: i64 = 4096;

struct Net {
    n: usize,
    w: Vec<i64>,
    b: Vec<i64>,
}

impl Net {
    fn w(&self, layer: usize, j: usize, i: usize) -> i64 {
        self.w[(layer * self.n + j) * self.n + i]
    }
}

fn mul(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

fn affine(net: &Net, l: usize, z: &mut [i64], q: &[i64]) {
    for j in 0..net.n {
        z[j] = z[j].wrapping_add(net.b[l * net.n + j]);
        for i in 0..net.n {
            z[j] = z[j].wrapping_add(mul(net.w(l, j, i), q[i]) / Q);
        }
    }
}

fn sigma(z: i64) -> i64 {
    mul(z, Q) / (Q + mul(z, z) / Q)
}

fn dsigma(z: i64) -> i64 {
    let zz = mul(z, z) / Q;
    mul(Q - zz, 16777216) / mul(Q + zz, Q + zz)
}

/// p += sign * c * W^T sigma(W q + b)
fn force(net: &Net, l: usize, p: &mut [i64], q: &[i64], c: i64, sign: i64) {
    let n = net.n;
    let mut z = vec![0; n];
    affine(net, l, &mut z, q);
    let s: Vec<i64> = z.iter().map(|&z| sigma(z)).collect();
    kick(net, l, p, &s, c, sign);
}

/// p += sign * c * W^T s
fn kick(net: &Net, l: usize, p: &mut [i64], s: &[i64], c: i64, sign: i64) {
    for i in 0..net.n {
        let mut t = 0i64;
        for j in 0..net.n {
            t = t.wrapping_add(mul(net.w(l, j, i), s[j]) / Q);
        }
        p[i] = p[i].wrapping_add(mul(sign, mul(c, t) / Q));
    }
}

fn drift(x: &mut [i64], y: &[i64], h: i64, sign: i64) {
    for i in 0..x.len() {
        x[i] = x[i].wrapping_add(mul(sign, mul(h, y[i]) / Q));
    }
}

fn layer(net: &Net, l: usize, q: &mut [i64], p: &mut [i64], h: i64) {
    let c = 0i64.wrapping_sub(h / 2);
    force(net, l, p, q, c, 1);
    drift(q, p, h, 1);
    force(net, l, p, q, c, 1);
}

fn vjp(net: &Net, l: usize, aq: &mut [i64], gw: &mut [i64], gb: &mut [i64], u: &[i64], q: &[i64], c: i64) {
    let n = net.n;
    let mut z = vec![0; n];
    affine(net, l, &mut z, q);
    let s: Vec<i64> = z.iter().map(|&z| sigma(z)).collect();
    let d: Vec<i64> = z.iter().map(|&z| dsigma(z)).collect();
    let mut t = vec![0i64; n];
    for j in 0..n {
        for i in 0..n {
            t[j] = t[j].wrapping_add(mul(net.w(l, j, i), u[i]) / Q);
        }
    }
    let dt: Vec<i64> = (0..n).map(|j| mul(d[j], t[j]) / Q).collect();
    kick(net, l, aq, &dt, c, 1);
    for j in 0..n {
        let cs = mul(c, s[j]) / Q;
        let cdt = mul(c, dt[j]) / Q;
        for i in 0..n {
            let at = (l * n + j) * n + i;
            gw[at] = gw[at].wrapping_add(mul(cs, u[i]) / Q).wrapping_add(mul(cdt, q[i]) / Q);
        }
        gb[l * n + j] = gb[l * n + j].wrapping_add(cdt);
    }
}

/// The adjoint of one layer from the states it ran through.
#[allow(clippy::too_many_arguments)]
fn layer_adjoint(net: &Net, l: usize, q0: &[i64], p0: &[i64], aq: &mut [i64], ap: &mut [i64], gw: &mut [i64], gb: &mut [i64], h: i64) {
    let c = 0i64.wrapping_sub(h / 2);
    let (mut q1, mut p1) = (q0.to_vec(), p0.to_vec());
    force(net, l, &mut p1, &q1, c, 1);
    drift(&mut q1, &p1, h, 1);
    vjp(net, l, aq, gw, gb, &ap.to_vec(), &q1, c);
    drift(ap, aq, h, 1);
    vjp(net, l, aq, gw, gb, &ap.to_vec(), q0, c);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (mode, n, layers, batch) = (args[1].as_str(), args[2].parse::<usize>().unwrap(), args[3].parse::<usize>().unwrap(), args[4].parse::<usize>().unwrap());
    let k = 4usize;
    let h = 1024i64;
    let mut net = Net { n, w: vec![0; layers * n * n], b: vec![0; layers * n] };
    for l in 0..layers {
        for j in 0..n {
            net.b[l * n + j] = ((j % 5) as i64 - 2) * 8;
            for i in 0..n {
                net.w[(l * n + j) * n + i] = (((l * 5 + j * 7 + i * 3) % 9) as i64 - 4) * 8;
            }
        }
    }
    let xs: Vec<Vec<i64>> = (0..batch).map(|s| (0..n).map(|i| (((s * 3 + i) % 7) as i64 - 3) * 512).collect()).collect();
    let ts: Vec<Vec<i64>> = (0..batch).map(|s| (0..k).map(|i| (((s + i) % 3) as i64 - 1) * 512).collect()).collect();
    let mut gw = vec![0i64; layers * n * n];
    let mut gb = vec![0i64; layers * n];
    let mut total = 0i64;
    let segment = (layers as f64).sqrt().ceil() as usize;
    let started = Instant::now();

    // The states each sample is in at the layers' inputs: all of them, or every segment-th.
    let keep_every = if mode == "stored" { 1 } else { segment };
    let mut states: Vec<Vec<(Vec<i64>, Vec<i64>)>> = Vec::new();
    let mut now: Vec<(Vec<i64>, Vec<i64>)> = xs.iter().map(|x| (x.clone(), vec![0; n])).collect();
    for l in 0..layers {
        if l % keep_every == 0 {
            states.push(now.clone());
        }
        for (q, p) in now.iter_mut() {
            layer(&net, l, q, p, h);
        }
    }
    let activation_bytes = states.len() * batch * 2 * n * 8;
    let mut aqs: Vec<Vec<i64>> = Vec::new();
    let mut aps: Vec<Vec<i64>> = Vec::new();
    for (s, (q, _)) in now.iter().enumerate() {
        for i in 0..k {
            total = total.wrapping_add(mul(q[i] - ts[s][i], q[i] - ts[s][i]) / 8192);
        }
        let mut aq = vec![0; n];
        for i in 0..k {
            aq[i] = q[i] - ts[s][i];
        }
        aqs.push(aq);
        aps.push(vec![0; n]);
    }
    // Backward, a segment at a time.
    let mut peak_segment_bytes = 0;
    let mut seg_start = ((layers - 1) / keep_every) * keep_every;
    loop {
        let seg_end = (seg_start + keep_every).min(layers);
        // recompute the states inside the segment from its stored input
        let mut inside: Vec<Vec<(Vec<i64>, Vec<i64>)>> = Vec::new();
        let mut cur = states[seg_start / keep_every].clone();
        for l in seg_start..seg_end {
            inside.push(cur.clone());
            for (q, p) in cur.iter_mut() {
                layer(&net, l, q, p, h);
            }
        }
        peak_segment_bytes = peak_segment_bytes.max(inside.len() * batch * 2 * n * 8);
        for l in (seg_start..seg_end).rev() {
            for s in 0..batch {
                let (q0, p0) = &inside[l - seg_start][s];
                layer_adjoint(&net, l, q0, p0, &mut aqs[s], &mut aps[s], &mut gw, &mut gb, h);
            }
        }
        if seg_start == 0 {
            break;
        }
        seg_start -= keep_every;
    }
    let elapsed = started.elapsed().as_secs_f64();
    let sum = |v: &[i64]| v.iter().fold(0i64, |a, &x| a.wrapping_mul(31).wrapping_add(x));
    println!("total {total} gw {} gb {}", sum(&gw), sum(&gb));
    let kept = if mode == "stored" { 0 } else { activation_bytes };
    eprintln!("activations: {} bytes kept, {} bytes recomputed at a time, {:.2} s", if mode == "stored" { activation_bytes } else { kept }, if mode == "stored" { 0 } else { peak_segment_bytes }, elapsed);
}
