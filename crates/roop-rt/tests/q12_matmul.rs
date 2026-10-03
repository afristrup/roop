//! The Q12 kernel returns what the loops of the `q12` einsums return, to the bit,
//! whatever the sizes and the magnitudes, and a negative sign takes it off again.

extern crate roop_rt;

unsafe extern "C" {
    fn roop_q12_matmul(
        c: *mut i64,
        a: *const i64,
        b: *const i64,
        sign: i64,
        m: i64,
        n: i64,
        k: i64,
        layout: i64,
    );
}

struct Random(u64);

impl Random {
    fn next(&mut self, range: i64) -> i64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % (2 * range as u64 + 1)) as i64 - range
    }

    fn fill(&mut self, len: usize, range: i64) -> Vec<i64> {
        (0..len).map(|_| self.next(range)).collect()
    }
}

fn loops(c: &mut [i64], a: &[i64], b: &[i64], dims: (usize, usize, usize), layout: i64) {
    let (m, n, k) = dims;
    for i in 0..m {
        for col in 0..n {
            for j in 0..k {
                let x = if layout == 2 {
                    a[j * m + i]
                } else {
                    a[i * k + j]
                };
                let y = if layout == 1 {
                    b[col * k + j]
                } else {
                    b[j * n + col]
                };
                c[i * n + col] = c[i * n + col].wrapping_add(x.wrapping_mul(y) / 4096);
            }
        }
    }
}

fn agrees(dims: (usize, usize, usize), range: i64, seed: u64) {
    let (m, n, k) = dims;
    let mut random = Random(seed);
    for layout in 0..3 {
        let a = random.fill(m * k, range);
        let b = random.fill(k * n, range);
        let start = random.fill(m * n, 1 << 20);
        let (mut expected, mut got) = (start.clone(), start.clone());
        loops(&mut expected, &a, &b, dims, layout);
        unsafe {
            roop_q12_matmul(
                got.as_mut_ptr(),
                a.as_ptr(),
                b.as_ptr(),
                1,
                m as i64,
                n as i64,
                k as i64,
                layout,
            );
        }
        assert_eq!(got, expected, "{dims:?} layout {layout} range {range}");
        unsafe {
            roop_q12_matmul(
                got.as_mut_ptr(),
                a.as_ptr(),
                b.as_ptr(),
                -1,
                m as i64,
                n as i64,
                k as i64,
                layout,
            );
        }
        assert_eq!(
            got, start,
            "{dims:?} layout {layout} range {range} backward"
        );
    }
}

#[test]
fn the_kernel_equals_the_loops_at_awkward_sizes() {
    for (seed, dims) in [
        (1, (1, 1, 1)),
        (2, (5, 7, 3)),
        (3, (4, 8, 64)),
        (4, (33, 65, 17)),
        (5, (32, 64, 64)),
        (6, (9, 100, 37)),
    ] {
        agrees(dims, 100_000, seed);
    }
}

#[test]
fn the_kernel_equals_the_loops_when_products_overflow_the_doubles() {
    for range in [1 << 24, 1 << 27, 1 << 40, i64::MAX] {
        agrees((9, 20, 24), range, 7);
    }
}

#[test]
fn the_kernel_equals_the_loops_at_the_edge_of_the_exact_range() {
    for range in [(1 << 25) - 1, 1 << 25, (1 << 25) + 1] {
        agrees((8, 16, 64), range, 8);
    }
}

#[test]
fn the_kernel_equals_the_loops_on_the_extremes() {
    let (m, n, k) = (8, 16, 20);
    let a = vec![i64::MIN; m * k];
    let b = vec![-1; k * n];
    let mut expected = vec![0; m * n];
    let mut got = vec![0; m * n];
    loops(&mut expected, &a, &b, (m, n, k), 0);
    unsafe {
        roop_q12_matmul(
            got.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            1,
            m as i64,
            n as i64,
            k as i64,
            0,
        );
    }
    assert_eq!(got, expected);
}
