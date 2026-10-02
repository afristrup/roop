mod support;

use roop_llvm::Options;

const N: usize = 4096; // 32 KiB per array: two whole 16 KiB pages

fn run(aligned: bool) -> i32 {
    let src = format!(
        "rev fn axpy(a: &mut [i64; {N}], b: &[i64; {N}], i: &mut i64, k: &i64) {{
            #[parallel(metal)] from i == 0 {{ a[i] += b[i] * k; }} loop {{ i += 1; }} until i == {};
        }}",
        N - 1
    );
    let out = support::compiled(&src, &Options::default());
    let lib = support::metallib(out.air.as_ref().unwrap()).unwrap();
    let host = roop_llvm::embed_blobs(&out.host, Some(&lib), None);
    let alloc = if aligned {
        format!("aligned_alloc(16384, {N} * 8)")
    } else {
        format!("malloc({N} * 8 + 16)")
    };
    let main = format!(
        r#"
#include <stdint.h>
#include <stdlib.h>
#define N {N}
int64_t roop_gpu_zero_copy_count(void);
void axpy(int64_t*, int64_t*, int64_t*, int64_t*);
void axpy_inv(int64_t*, int64_t*, int64_t*, int64_t*);
int main(void) {{
    int64_t *a = {alloc}, *b = {alloc};
    int64_t i = 0, k = 3;
    for (int64_t m = 0; m < N; m++) {{ a[m] = m; b[m] = 2 * m; }}
    axpy(a, b, &i, &k);
    for (int64_t m = 0; m < N; m++) if (a[m] != 7 * m) return 1;
    axpy_inv(a, b, &i, &k);
    for (int64_t m = 0; m < N; m++) if (a[m] != m) return 2;
    return (int)roop_gpu_zero_copy_count();
}}
"#
    );
    let files = [
        ("m0.ll".to_string(), host.as_str()),
        ("main.c".to_string(), main.as_str()),
    ];
    support::run_native_files(&files).unwrap().unwrap()
}

fn metal_available() -> bool {
    cfg!(target_os = "macos")
        && std::process::Command::new("xcrun")
            .args(["-sdk", "macosx", "--find", "metal"])
            .output()
            .is_ok_and(|o| o.status.success())
}

#[test]
fn page_aligned_arrays_are_shared_with_the_gpu_without_copying() {
    if !metal_available() {
        return;
    }
    // Two launches (forward and inverse), two arrays each.
    assert_eq!(run(true), 4);
}

#[test]
fn unaligned_arrays_still_work_through_pooled_staging_buffers() {
    if !metal_available() {
        return;
    }
    assert_eq!(run(false), 0);
}
