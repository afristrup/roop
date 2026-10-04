//! The integer kernel returns what the loops of the `i64` einsums return, to the
//! bit, whatever the sizes and the magnitudes, and a negative sign takes it off
//! again.

extern crate roop_rt;

unsafe extern "C" {
    fn roop_i64_matmul(
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

/// The kernel's prologue calls this only when a caller has a lazy save of ZA
/// pending, which a Rust test never has; clang's runtime defines it for programs.
#[unsafe(no_mangle)]
extern "C" fn __arm_tpidr2_save() {}

struct Random(u64);

impl Random {
    fn next(&mut self, range: i64) -> i64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        if range == i64::MAX {
            return self.0 as i64;
        }
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
                let x = if layout == 2 { a[j * m + i] } else { a[i * k + j] };
                let y = if layout == 1 { b[col * k + j] } else { b[j * n + col] };
                c[i * n + col] = c[i * n + col].wrapping_add(x.wrapping_mul(y));
            }
        }
    }
}

fn call(c: &mut [i64], a: &[i64], b: &[i64], sign: i64, dims: (usize, usize, usize), layout: i64) {
    let (m, n, k) = dims;
    unsafe {
        roop_i64_matmul(
            c.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            sign,
            m as i64,
            n as i64,
            k as i64,
            layout,
        );
    }
}

fn agrees(dims: (usize, usize, usize), range: i64, seed: u64) {
    let (m, n, k) = dims;
    let mut random = Random(seed);
    for layout in 0..3 {
        let a = random.fill(m * k, range);
        let b = random.fill(k * n, range);
        let start = random.fill(m * n, 1 << 40);
        let (mut expected, mut got) = (start.clone(), start.clone());
        loops(&mut expected, &a, &b, dims, layout);
        call(&mut got, &a, &b, 1, dims, layout);
        assert_eq!(got, expected, "{dims:?} layout {layout} range {range}");
        call(&mut got, &a, &b, -1, dims, layout);
        assert_eq!(got, start, "{dims:?} layout {layout} range {range} backward");
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
        (7, (17, 31, 300)),
        (8, (130, 45, 33)),
    ] {
        agrees(dims, 100_000, seed);
    }
}

#[test]
fn the_kernel_equals_the_loops_beyond_a_block_of_rows() {
    agrees((1100, 20, 24), 50_000, 9);
}

#[test]
fn the_kernel_equals_the_loops_on_the_threads() {
    agrees((130, 520, 1100), 1 << 10, 10);
}

#[test]
fn the_kernel_equals_the_loops_when_products_overflow_the_doubles() {
    for range in [1 << 26, 1 << 30, 1 << 40, i64::MAX] {
        agrees((9, 20, 24), range, 11);
    }
}

#[test]
fn the_kernel_equals_the_loops_at_the_edge_of_the_exact_range() {
    for range in [(1 << 23) - 1, 1 << 23, (1 << 23) + 1] {
        agrees((8, 16, 64), range, 12);
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
    call(&mut got, &a, &b, 1, (m, n, k), 0);
    assert_eq!(got, expected);
}
