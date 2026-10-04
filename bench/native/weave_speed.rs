// Hand-written baselines for weave's training step: the same leapfrog network
// (8 layers of width 64, the cauchy force, 32 samples) trained the usual way,
// with the backward pass reading the states the forward pass stored, in four
// ways, and timed on the same data as roop/weave's benchmark:
//
//     q12        integers on the 1/4096 grid, term for term the arithmetic of
//                roop/weave, loops written the way idiomatic Rust writes them
//     f64        the same network in doubles, the same loops
//     accelerate the doubles again, with every product a cblas_dgemm
//
// Each runs one sample at a time, the whole batch as matrix products, and the
// batch split over threads, the way a framework would. `weave_speed N` prints
// samples a second, and a checksum of the weights after the steps.
use std::time::Instant;

#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn cblas_dgemm(
        order: i32, ta: i32, tb: i32, m: i32, n: i32, k: i32, alpha: f64,
        a: *const f64, lda: i32, b: *const f64, ldb: i32, beta: f64, c: *mut f64, ldc: i32,
    );
}

trait Num: Copy + Send + Sync + 'static {
    const ZERO: Self;
    fn add(self, o: Self) -> Self;
    fn sub(self, o: Self) -> Self;
    fn mul(self, o: Self) -> Self;
    /// The product of two numbers as a sum adds it, not yet divided by 4096.
    fn wide(self, o: Self) -> Self;
    /// A sum of wide products, back on the grid.
    fn narrow(self) -> Self;
    /// w - lr * g, with g a gradient of a matrix, in Q24 for integers.
    fn update(self, lr: Self, g: Self) -> Self;
    fn half_square(self) -> Self;
    fn sigma(self) -> Self;
    fn dsigma(self) -> Self;
    fn halve(self) -> Self;
    fn neg(self) -> Self;
    fn checksum(self) -> i64;
}

impl Num for i64 {
    const ZERO: i64 = 0;
    fn add(self, o: i64) -> i64 { self.wrapping_add(o) }
    fn sub(self, o: i64) -> i64 { self.wrapping_sub(o) }
    fn mul(self, o: i64) -> i64 { self.wrapping_mul(o) / 4096 }
    fn wide(self, o: i64) -> i64 { self.wrapping_mul(o) }
    fn narrow(self) -> i64 { self / 4096 }
    fn update(self, lr: i64, g: i64) -> i64 { self.wrapping_sub(lr.wrapping_mul(g) / 16777216) }
    fn half_square(self) -> i64 { self.wrapping_mul(self) / 8192 }
    fn sigma(self) -> i64 { self.wrapping_mul(4096) / (4096 + self.wrapping_mul(self) / 4096) }
    fn dsigma(self) -> i64 {
        let zz = self.wrapping_mul(self) / 4096;
        (4096 - zz).wrapping_mul(16777216) / (4096 + zz).wrapping_mul(4096 + zz)
    }
    fn halve(self) -> i64 { self / 2 }
    fn neg(self) -> i64 { self.wrapping_neg() }
    fn checksum(self) -> i64 { self }
}

impl Num for f64 {
    const ZERO: f64 = 0.0;
    fn add(self, o: f64) -> f64 { self + o }
    fn sub(self, o: f64) -> f64 { self - o }
    fn mul(self, o: f64) -> f64 { self * o }
    fn wide(self, o: f64) -> f64 { self * o }
    fn narrow(self) -> f64 { self }
    fn update(self, lr: f64, g: f64) -> f64 { self - lr * g }
    fn half_square(self) -> f64 { self * self / 2.0 }
    fn sigma(self) -> f64 { self / (1.0 + self * self) }
    fn dsigma(self) -> f64 {
        let zz = self * self;
        (1.0 - zz) / ((1.0 + zz) * (1.0 + zz))
    }
    fn halve(self) -> f64 { self / 2.0 }
    fn neg(self) -> f64 { -self }
    fn checksum(self) -> i64 { (self * 4096.0).round() as i64 }
}

/// The three products of a layer, on row-major matrices of B samples.
trait Gemm<T: Num>: Sync {
    /// out[b][m] += (sum_n x[b][n] w[m][n]) / 4096
    fn nt(out: &mut [T], x: &[T], w: &[T], b: usize, n: usize, m: usize);
    /// out[b][n] += (sum_m s[b][m] w[m][n]) / 4096
    fn nn(out: &mut [T], s: &[T], w: &[T], b: usize, m: usize, n: usize);
    /// out[m][n] += sum_b a[b][m] c[b][n], not divided by 4096: a gradient in Q24
    fn tn(out: &mut [T], a: &[T], c: &[T], b: usize, m: usize, n: usize);
}

struct Loops;

impl<T: Num> Gemm<T> for Loops {
    fn nt(out: &mut [T], x: &[T], w: &[T], _b: usize, n: usize, m: usize) {
        for (row, o) in x.chunks(n).zip(out.chunks_mut(m)) {
            for (cell, wr) in o.iter_mut().zip(w.chunks(n)) {
                let sum = row.iter().zip(wr).fold(T::ZERO, |s, (a, b)| s.add(a.wide(*b)));
                *cell = cell.add(sum.narrow());
            }
        }
    }
    fn nn(out: &mut [T], s: &[T], w: &[T], _b: usize, m: usize, n: usize) {
        for (row, o) in s.chunks(m).zip(out.chunks_mut(n)) {
            let mut sums = vec![T::ZERO; n];
            for (sj, wr) in row.iter().zip(w.chunks(n)) {
                for (cell, wv) in sums.iter_mut().zip(wr) {
                    *cell = cell.add(sj.wide(*wv));
                }
            }
            for (cell, sum) in o.iter_mut().zip(sums) {
                *cell = cell.add(sum.narrow());
            }
        }
    }
    fn tn(out: &mut [T], a: &[T], c: &[T], _b: usize, m: usize, n: usize) {
        for (arow, crow) in a.chunks(m).zip(c.chunks(n)) {
            for (ai, o) in arow.iter().zip(out.chunks_mut(n)) {
                for (cell, cv) in o.iter_mut().zip(crow) {
                    *cell = cell.add(ai.wide(*cv));
                }
            }
        }
    }
}

struct Blas;

fn dgemm(ta: i32, tb: i32, m: usize, n: usize, k: usize, a: &[f64], lda: usize, b: &[f64], ldb: usize, c: &mut [f64]) {
    unsafe {
        cblas_dgemm(101, ta, tb, m as i32, n as i32, k as i32, 1.0, a.as_ptr(), lda as i32, b.as_ptr(), ldb as i32, 1.0, c.as_mut_ptr(), n as i32)
    }
}

impl Gemm<f64> for Blas {
    fn nt(out: &mut [f64], x: &[f64], w: &[f64], b: usize, n: usize, m: usize) {
        dgemm(111, 112, b, m, n, x, n, w, n, out);
    }
    fn nn(out: &mut [f64], s: &[f64], w: &[f64], b: usize, m: usize, n: usize) {
        dgemm(111, 111, b, n, m, s, m, w, n, out);
    }
    fn tn(out: &mut [f64], a: &[f64], c: &[f64], b: usize, m: usize, n: usize) {
        dgemm(112, 111, m, n, b, a, m, c, n, out);
    }
}

struct Shape {
    n: usize,
    layers: usize,
    outputs: usize,
}

#[derive(Clone)]
struct Params<T> {
    w: Vec<Vec<T>>,
    b: Vec<Vec<T>>,
}

impl<T: Num> Params<T> {
    fn zeros(shape: &Shape) -> Self {
        Params { w: vec![vec![T::ZERO; shape.n * shape.n]; shape.layers], b: vec![vec![T::ZERO; shape.n]; shape.layers] }
    }
}

fn scaled<T: Num>(v: &mut [T], c: T, y: &[T]) {
    for (x, y) in v.iter_mut().zip(y) {
        *x = x.add(c.mul(*y));
    }
}

/// z = W q + b, s = f(z) for each row
fn affine<T: Num, G: Gemm<T>>(w: &[T], bias: &[T], q: &[T], rows: usize, n: usize) -> Vec<T> {
    let mut z: Vec<T> = bias.iter().copied().cycle().take(rows * n).collect();
    G::nt(&mut z, q, w, rows, n, n);
    z
}

/// p += c W^T f(W q + b)
fn force<T: Num, G: Gemm<T>>(w: &[T], bias: &[T], p: &mut [T], q: &[T], c: T, rows: usize, n: usize) {
    let s: Vec<T> = affine::<T, G>(w, bias, q, rows, n).into_iter().map(T::sigma).collect();
    let mut t = vec![T::ZERO; rows * n];
    G::nn(&mut t, &s, w, rows, n, n);
    scaled(p, c, &t);
}

/// aq += c W^T (f' * (W u)), gw += c (f u^T + (f' * W u) q^T), gb += c (f' * W u)
#[allow(clippy::too_many_arguments)]
fn vjp<T: Num, G: Gemm<T>>(w: &[T], bias: &[T], aq: &mut [T], gw: &mut [T], gb: &mut [T], u: &[T], q: &[T], c: T, rows: usize, n: usize) {
    let z = affine::<T, G>(w, bias, q, rows, n);
    let s: Vec<T> = z.iter().map(|z| z.sigma()).collect();
    let d: Vec<T> = z.iter().map(|z| z.dsigma()).collect();
    let mut t = vec![T::ZERO; rows * n];
    G::nt(&mut t, u, w, rows, n, n);
    let dt: Vec<T> = d.iter().zip(&t).map(|(d, t)| d.mul(*t)).collect();
    let mut back = vec![T::ZERO; rows * n];
    G::nn(&mut back, &dt, w, rows, n, n);
    scaled(aq, c, &back);
    let cs: Vec<T> = s.iter().map(|s| c.mul(*s)).collect();
    let cdt: Vec<T> = dt.iter().map(|d| c.mul(*d)).collect();
    G::tn(gw, &cs, u, rows, n, n);
    G::tn(gw, &cdt, q, rows, n, n);
    for row in cdt.chunks(n) {
        for (g, x) in gb.iter_mut().zip(row) {
            *g = g.add(*x);
        }
    }
}

/// The loss of a chunk of samples, and the sum of their gradients added to `grads`.
fn grad<T: Num, G: Gemm<T>>(net: &Params<T>, shape: &Shape, h: T, xs: &[T], ts: &[T], grads: &mut Params<T>) -> T {
    let (n, rows) = (shape.n, xs.len() / shape.n);
    let c = h.halve().neg();
    let (mut q, mut p) = (xs.to_vec(), vec![T::ZERO; xs.len()]);
    let mut stored: Vec<(Vec<T>, Vec<T>)> = Vec::with_capacity(shape.layers);
    for l in 0..shape.layers {
        stored.push((q.clone(), p.clone()));
        force::<T, G>(&net.w[l], &net.b[l], &mut p, &q, c, rows, n);
        scaled(&mut q, h, &p);
        force::<T, G>(&net.w[l], &net.b[l], &mut p, &q, c, rows, n);
    }
    let mut total = T::ZERO;
    let mut aq = vec![T::ZERO; xs.len()];
    for r in 0..rows {
        for k in 0..shape.outputs {
            let d = q[r * n + k].sub(ts[r * shape.outputs + k]);
            total = total.add(d.half_square());
            aq[r * n + k] = d;
        }
    }
    let mut ap = vec![T::ZERO; xs.len()];
    for l in (0..shape.layers).rev() {
        let (q0, p0) = &stored[l];
        let mut p1 = p0.clone();
        force::<T, G>(&net.w[l], &net.b[l], &mut p1, q0, c, rows, n);
        let mut q1 = q0.clone();
        scaled(&mut q1, h, &p1);
        let (gw, gb) = (&mut grads.w[l], &mut grads.b[l]);
        vjp::<T, G>(&net.w[l], &net.b[l], &mut aq, gw, gb, &ap, &q1, c, rows, n);
        let step = aq.clone();
        scaled(&mut ap, h, &step);
        vjp::<T, G>(&net.w[l], &net.b[l], &mut aq, gw, gb, &ap, q0, c, rows, n);
    }
    total
}

/// One step: the samples in `chunks` pieces, each on a thread when there are several.
fn step<T: Num, G: Gemm<T>>(net: &mut Params<T>, shape: &Shape, h: T, lr: T, xs: &[T], ts: &[T], chunks: usize) -> T {
    let rows = xs.len() / shape.n;
    let size = rows.div_ceil(chunks);
    let work = |x: &[T], t: &[T]| {
        let mut g = Params::zeros(shape);
        let total = grad::<T, G>(net, shape, h, x, t, &mut g);
        (total, g)
    };
    let pieces = xs.chunks(size * shape.n).zip(ts.chunks(size * shape.outputs));
    let results: Vec<(T, Params<T>)> = if chunks == 1 {
        pieces.map(|(x, t)| work(x, t)).collect()
    } else {
        std::thread::scope(|s| {
            let handles: Vec<_> = pieces.map(|(x, t)| s.spawn({ let work = &work; move || work(x, t) })).collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        })
    };
    let mut sum: Params<T> = Params::zeros(shape);
    let mut total = T::ZERO;
    for (t, g) in results {
        total = total.add(t);
        for l in 0..shape.layers {
            sum.w[l].iter_mut().zip(&g.w[l]).for_each(|(a, b)| *a = a.add(*b));
            sum.b[l].iter_mut().zip(&g.b[l]).for_each(|(a, b)| *a = a.add(*b));
        }
    }
    for l in 0..shape.layers {
        net.w[l].iter_mut().zip(&sum.w[l]).for_each(|(w, g)| *w = w.update(lr, *g));
        net.b[l].iter_mut().zip(&sum.b[l]).for_each(|(b, g)| *b = b.sub(lr.mul(*g)));
    }
    total
}

fn best_of(rounds: usize, mut f: impl FnMut()) -> f64 {
    (0..rounds)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64()
        })
        .skip(1)
        .fold(f64::INFINITY, f64::min)
}

/// The benchmark's data, on the grid, as numbers of `T` (`scale` is 4096 for Q12 and 1 for doubles).
fn data<T: Num>(shape: &Shape, samples: usize, make: impl Fn(i64) -> T) -> (Params<T>, Vec<T>, Vec<T>) {
    let mut net = Params::<T>::zeros(shape);
    for l in 0..shape.layers {
        for j in 0..shape.n {
            net.b[l][j] = make((j as i64 % 5 - 2) * 64);
            for i in 0..shape.n {
                net.w[l][j * shape.n + i] = make(((l * 5 + j * 7 + i * 3) as i64 % 9 - 4) * 64);
            }
        }
    }
    let xs = (0..samples).flat_map(|s| (0..shape.n).map(move |i| ((s * 3 + i) as i64 % 7 - 3) * 512)).map(&make).collect();
    let ts = (0..samples).flat_map(|s| (0..shape.outputs).map(move |k| ((s + k) as i64 % 3 - 1) * 512)).map(&make).collect();
    (net, xs, ts)
}

fn env_or(name: &str, default: usize) -> usize {
    std::env::var(name).map_or(default, |v| v.parse().unwrap())
}

fn run<T: Num, G: Gemm<T>>(label: &str, make: impl Fn(i64) -> T + Copy, per_sample: bool, chunks: usize, steps: usize) {
    let shape = Shape { n: env_or("WEAVE_WIDTH", 64), layers: 8, outputs: 4 };
    let samples = env_or("WEAVE_SAMPLES", 32);
    let (h, lr) = (make(1024), make(64));
    let mut checksum = 0;
    let secs = best_of(7, || {
        let (mut net, xs, ts) = data::<T>(&shape, samples, make);
        for _ in 0..steps {
            if per_sample {
                // one sample at a time, the gradients summed as the step does
                let mut g: Params<T> = Params::zeros(&shape);
                for (x, t) in xs.chunks(shape.n).zip(ts.chunks(shape.outputs)) {
                    grad::<T, G>(&net, &shape, h, x, t, &mut g);
                }
                for l in 0..shape.layers {
                    net.w[l].iter_mut().zip(&g.w[l]).for_each(|(w, g)| *w = w.update(lr, *g));
                    net.b[l].iter_mut().zip(&g.b[l]).for_each(|(b, g)| *b = b.sub(lr.mul(*g)));
                }
            } else {
                step::<T, G>(&mut net, &shape, h, lr, &xs, &ts, chunks);
            }
        }
        checksum = net.w.iter().flatten().chain(net.b.iter().flatten()).fold(0i64, |a, x| a.wrapping_mul(31).wrapping_add(x.checksum()));
    });
    println!("{label}: {:.1} samples/s, {:.2} ms a step (weights checksum {checksum})", (steps * samples) as f64 / secs, 1000.0 * secs / steps as f64);
}

fn main() {
    let steps: usize = std::env::args().nth(1).map_or(5, |s| s.parse().unwrap());
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let q12 = |x: i64| x;
    let real = |x: i64| x as f64 / 4096.0;
    run::<i64, Loops>("Rust q12, one sample at a time", q12, true, 1, steps);
    run::<i64, Loops>("Rust q12, the batch as matrices", q12, false, 1, steps);
    run::<i64, Loops>(&format!("Rust q12, the batch in {threads} chunks"), q12, false, threads, steps);
    run::<f64, Loops>("Rust f64, one sample at a time", real, true, 1, steps);
    run::<f64, Loops>("Rust f64, the batch as matrices", real, false, 1, steps);
    run::<f64, Loops>(&format!("Rust f64, the batch in {threads} chunks"), real, false, threads, steps);
    run::<f64, Blas>("Accelerate f64, the batch as dgemm", real, false, 1, steps);
    run::<f64, Blas>(&format!("Accelerate f64, the batch in {threads} chunks"), real, false, threads, steps);
}
