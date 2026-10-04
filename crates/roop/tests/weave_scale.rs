mod support;

use std::path::PathBuf;
use support::{project, roop, stderr};

fn config() -> String {
    let weave = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/weave");
    format!(
        "[modules]\nweave = \"{}\"\neinsum = \"{}\"\n",
        weave.display(),
        weave.with_file_name("einsum").display()
    )
}

/// The forward pass, the backward pass and the gradient of a network of width
/// `n`, hidden width `m`, `l` layers and `k` outputs, as one program.
fn program(n: usize, m: usize, l: usize, k: usize) -> String {
    format!(
        "
use weave::forward;
use weave::backward;
use weave::grad;

fn fwd(q: &mut [i64; {n}], p: &mut [i64; {n}], ws: &[[[i64; {n}]; {m}]; {l}], bs: &[[i64; {m}]; {l}], h: &i64, kind: &i64) {{
    call forward<{n}, {m}, {l}>(q, p, ws, bs, h, kind);
}}

fn bwd(q: &mut [i64; {n}], p: &mut [i64; {n}], aq: &mut [i64; {n}], ap: &mut [i64; {n}],
       gw: &mut [[[i64; {n}]; {m}]; {l}], gb: &mut [[i64; {m}]; {l}],
       ws: &[[[i64; {n}]; {m}]; {l}], bs: &[[i64; {m}]; {l}], h: &i64, kind: &i64) {{
    call backward<{n}, {m}, {l}>(q, p, aq, ap, gw, gb, ws, bs, h, kind);
}}

fn g(total: &mut i64, q: &mut [i64; {n}], p: &mut [i64; {n}], aq: &mut [i64; {n}], ap: &mut [i64; {n}],
     gw: &mut [[[i64; {n}]; {m}]; {l}], gb: &mut [[i64; {m}]; {l}],
     ws: &[[[i64; {n}]; {m}]; {l}], bs: &[[i64; {m}]; {l}], h: &i64, kind: &i64, t: &[i64; {k}]) {{
    call grad<{n}, {m}, {l}, {k}>(total, q, p, aq, ap, gw, gb, ws, bs, h, kind, t);
}}
"
    )
}

fn lean_proves(name: &str, n: usize, m: usize, l: usize, k: usize) {
    let dir = project(name, &config(), &program(n, m, l, k));
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for function in ["fwd", "bwd", "g"] {
        assert!(report.contains(function), "{function} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

#[test]
fn lean_proves_forward_backward_and_grad_at_width_16_with_4_layers() {
    lean_proves("weave-scale-16", 16, 16, 4, 4);
}

#[test]
fn lean_proves_forward_backward_and_grad_at_width_32_with_8_layers() {
    lean_proves("weave-scale-32", 32, 32, 8, 4);
}

/// The size of the benchmark. About a minute and a half on its own.
#[test]
fn lean_proves_forward_backward_and_grad_at_the_benchmark_size() {
    lean_proves("weave-scale-64", 64, 64, 8, 4);
}
