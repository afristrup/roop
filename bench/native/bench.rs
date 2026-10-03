// Hand-written baselines: the same algorithms as roop's, on one thread and on
// all of them, and Apple's Accelerate where it has a routine for it.
use std::time::Instant;

#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn cblas_dgemm(
        order: i32, ta: i32, tb: i32, m: i32, n: i32, k: i32, alpha: f64,
        a: *const f64, lda: i32, b: *const f64, ldb: i32, beta: f64, c: *mut f64, ldc: i32,
    );
    fn cblas_daxpy(n: i32, alpha: f64, x: *const f64, incx: i32, y: *mut f64, incy: i32);
}

fn best(times: Vec<f64>) -> f64 {
    times.into_iter().fold(f64::INFINITY, f64::min)
}

fn time(reps: usize, mut f: impl FnMut()) -> f64 {
    time_batched(reps, 1, &mut f)
}

fn time_batched(reps: usize, inner: usize, f: &mut impl FnMut()) -> f64 {
    for _ in 0..inner {
        f();
    }
    best((0..reps).map(|_| {
        let t = Instant::now();
        for _ in 0..inner {
            f();
        }
        t.elapsed().as_secs_f64() / inner as f64
    }).collect())
}

fn gemm_rows(c: &mut [f64], a: &[f64], b: &[f64], n: usize, first: usize) {
    for (r, row) in c.chunks_mut(n).enumerate() {
        let i = first + r;
        let a_row = &a[i * n..(i + 1) * n];
        for (j, cell) in row.iter_mut().enumerate() {
            let column = b[j..].iter().step_by(n);
            *cell += a_row.iter().zip(column).map(|(x, y)| x * y).sum::<f64>();
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let kind = args[1].as_str();
    let n: usize = args[2].parse().unwrap();
    let reps: usize = args[3].parse().unwrap();
    let threads = std::thread::available_parallelism().unwrap().get();
    match kind {
        "gemm" => {
            let a: Vec<f64> = (0..n * n).map(|i| (i % 7) as f64 * 0.25).collect();
            let b: Vec<f64> = (0..n * n).map(|i| (i % 5) as f64 * 0.5).collect();
            let mut c = vec![0.0; n * n];
            let one = time(reps, || gemm_rows(&mut c, &a, &b, n, 0));
            let many = time(reps, || {
                let per = n.div_ceil(threads);
                std::thread::scope(|s| {
                    for (k, chunk) in c.chunks_mut(per * n).enumerate() {
                        let (a, b) = (&a, &b);
                        s.spawn(move || gemm_rows(chunk, a, b, n, k * per));
                    }
                });
            });
            let blas = time(reps, || unsafe {
                cblas_dgemm(101, 111, 111, n as i32, n as i32, n as i32, 1.0, a.as_ptr(), n as i32, b.as_ptr(), n as i32, 1.0, c.as_mut_ptr(), n as i32)
            });
            println!("{one:.9} {many:.9} {blas:.9}");
        }
        _ => {
            let x: Vec<f64> = (0..n).map(|i| (i % 7) as f64 * 0.25).collect();
            let mut y = vec![1.0; n];
            let inner = if n < 100_000 { 400_000 / n } else { 1 };
            let one = time_batched(reps, inner, &mut || {
                for (a, b) in y.iter_mut().zip(&x) {
                    *a += 1.0 * b;
                }
            });
            let many = time_batched(reps, inner, &mut || {
                let per = n.div_ceil(threads);
                std::thread::scope(|s| {
                    for (k, chunk) in y.chunks_mut(per).enumerate() {
                        let x = &x;
                        s.spawn(move || {
                            for (v, w) in chunk.iter_mut().zip(&x[k * per..]) {
                                *v += 1.0 * w;
                            }
                        });
                    }
                });
            });
            let blas = time_batched(reps, inner, &mut || unsafe {
                cblas_daxpy(n as i32, 1.0, x.as_ptr(), 1, y.as_mut_ptr(), 1)
            });
            println!("{one:.9} {many:.9} {blas:.9}");
        }
    }
}
