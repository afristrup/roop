#![allow(dead_code)]

pub const N: usize = 1000;

pub fn program(target: &str) -> String {
    format!(
        "rev fn two(a: &mut [i64; {N}], b: &mut [i64; {N}], i: &mut i64, j: &mut i64, k: &i64) {{
            #[parallel({target})] from i == 0 {{ a[i] += b[i] * k; }} loop {{ i += 1; }} until i == {last};
            #[parallel({target})] from j == 0 {{ b[j] += a[j]; }} loop {{ j += 1; }} until j == {last};
        }}",
        last = N - 1
    )
}

/// a[m] = m, b[m] = 2m, k = 3. After `two`: a = 7m, b = 9m, i = j = N-1.
/// Running `two_inv` must restore everything, induction variables included.
pub const HARNESS: &str = r#"
#include <stdint.h>
#define N 1000
void two(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void two_inv(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
static int64_t a[N], b[N];
int main(void) {
    int64_t i = 0, j = 0, k = 3;
    for (int64_t m = 0; m < N; m++) { a[m] = m; b[m] = 2 * m; }
    two(a, b, &i, &j, &k);
    for (int64_t m = 0; m < N; m++) {
        if (a[m] != 7 * m || b[m] != 9 * m) return 1;
    }
    if (i != N - 1 || j != N - 1) return 2;
    two_inv(a, b, &i, &j, &k);
    for (int64_t m = 0; m < N; m++) {
        if (a[m] != m || b[m] != 2 * m) return 3;
    }
    return (i != 0 || j != 0) ? 4 : 0;
}
"#;
