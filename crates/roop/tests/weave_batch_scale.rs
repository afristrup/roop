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

fn lean_proves(name: &str, src: &str, names: &[&str]) {
    let dir = project(name, &config(), src);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in names {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

/// 32 rows, width 64, hidden width 64, 20 chain cells. About ten seconds.
#[test]
fn lean_proves_the_batched_residual_block_at_32_rows_width_64_and_20_cells() {
    let (b, n, m, k) = (32, 64, 64, 20);
    let src = format!(
        "
use weave::residual_batch;
use weave::residual_back_batch;

fn fwd(q: &mut [[i64; {n}]; {b}], p: &mut [[i64; {n}]; {b}], w1: &[[i64; {n}]; {m}], b1: &[i64; {m}],
       w2: &[[i64; {m}]; {n}], b2: &[i64; {n}], kind: &i64) {{
    call residual_batch<{n}, {m}, {k}, {b}>(q, p, w1, b1, w2, b2, kind);
}}
fn back(q: &mut [[i64; {n}]; {b}], p: &mut [[i64; {n}]; {b}], aq: &mut [[i64; {n}]; {b}], ap: &mut [[i64; {n}]; {b}],
        gw1: &mut [[i64; {n}]; {m}], gb1: &mut [i64; {m}], gw2: &mut [[i64; {m}]; {n}], gb2: &mut [i64; {n}],
        w1: &[[i64; {n}]; {m}], b1: &[i64; {m}], w2: &[[i64; {m}]; {n}], b2: &[i64; {n}], kind: &i64) {{
    call residual_back_batch<{n}, {m}, {k}, {b}>(q, p, aq, ap, gw1, gb1, gw2, gb2, w1, b1, w2, b2, kind);
}}
"
    );
    lean_proves(
        "weave-batch-scale-residual",
        &src,
        &[
            "weave__residual_batch__residual_batch__64_64_20_32",
            "weave__residual_back_batch__residual_back_batch__64_64_20_32",
        ],
    );
}

/// 32 rows, 16 channels, kernel 3, 32 steps. About fifteen seconds.
#[test]
fn lean_proves_the_batched_convolution_at_16_channels_and_32_steps() {
    let (b, c, ks, t) = (32, 16, 3, 32);
    let (n, ck, bt) = (c * t, c * ks, b * t);
    let src = format!(
        "
use weave::conv_batch;
use weave::conv_back_batch;

fn fwd(y: &mut [[i64; {n}]; {b}], w: &[[i64; {ck}]; {c}], b: &[i64; {c}], x: &[[i64; {n}]; {b}]) {{
    call conv_batch<{c}, {ks}, {t}, {n}, {ck}, {b}, {bt}>(y, w, b, x);
}}
fn back(y: &mut [[i64; {n}]; {b}], ay: &[[i64; {n}]; {b}], ax: &mut [[i64; {n}]; {b}],
        gw: &mut [[i64; {ck}]; {c}], gb: &mut [i64; {c}],
        w: &[[i64; {ck}]; {c}], b: &[i64; {c}], x: &[[i64; {n}]; {b}]) {{
    call conv_back_batch<{c}, {ks}, {t}, {n}, {ck}, {b}, {bt}>(y, ay, ax, gw, gb, w, b, x);
}}
"
    );
    lean_proves(
        "weave-batch-scale-conv",
        &src,
        &["weave__conv_batch__conv_batch__16_3_32_512_48_32_1024"],
    );
}

/// 32 rows, width 64, hidden width 64, RMS normalized. About a minute.
#[test]
fn lean_proves_the_batched_rms_normalized_perceptron_at_width_64() {
    let (b, n, m) = (32, 64, 64);
    let src = format!(
        "
use weave::mlp_norm_batch;
use weave::mlp_norm_back_batch;

fn fwd(y: &mut [[i64; {n}]; {b}], w1: &[[i64; {n}]; {m}], b1: &[i64; {m}], g: &[i64; {m}], w2: &[[i64; {m}]; {n}],
       b2: &[i64; {n}], x: &[[i64; {n}]; {b}], kind: &i64, eps: &i64) {{
    call mlp_norm_batch<{n}, {m}, {b}>(y, w1, b1, g, w2, b2, x, kind, eps);
}}
fn back(y: &mut [[i64; {n}]; {b}], ay: &[[i64; {n}]; {b}], ax: &mut [[i64; {n}]; {b}],
        gw1: &mut [[i64; {n}]; {m}], gb1: &mut [i64; {m}], gg: &mut [i64; {m}],
        gw2: &mut [[i64; {m}]; {n}], gb2: &mut [i64; {n}],
        w1: &[[i64; {n}]; {m}], b1: &[i64; {m}], g: &[i64; {m}], w2: &[[i64; {m}]; {n}], b2: &[i64; {n}],
        x: &[[i64; {n}]; {b}], kind: &i64, eps: &i64) {{
    call mlp_norm_back_batch<{n}, {m}, {b}>(y, ay, ax, gw1, gb1, gg, gw2, gb2, w1, b1, g, w2, b2, x, kind, eps);
}}
"
    );
    lean_proves(
        "weave-batch-scale-mlp-norm",
        &src,
        &[
            "weave__mlp_norm_batch__mlp_norm_batch__64_64_32",
            "weave__mlp_norm_batch__mlp_norm_back_batch__64_64_32",
        ],
    );
}

/// 32 rows, width 64, hidden width 64, layer normalized. About a minute.
#[test]
fn lean_proves_the_batched_layer_normalized_perceptron_at_width_64() {
    let (b, n, m) = (32, 64, 64);
    let src = format!(
        "
use weave::mlp_layer_batch;
use weave::mlp_layer_back_batch;

fn fwd(y: &mut [[i64; {n}]; {b}], w1: &[[i64; {n}]; {m}], b1: &[i64; {m}], g: &[i64; {m}], beta: &[i64; {m}],
       w2: &[[i64; {m}]; {n}], b2: &[i64; {n}], x: &[[i64; {n}]; {b}], kind: &i64, eps: &i64) {{
    call mlp_layer_batch<{n}, {m}, {b}>(y, w1, b1, g, beta, w2, b2, x, kind, eps);
}}
fn back(y: &mut [[i64; {n}]; {b}], ay: &[[i64; {n}]; {b}], ax: &mut [[i64; {n}]; {b}],
        gw1: &mut [[i64; {n}]; {m}], gb1: &mut [i64; {m}], gg: &mut [i64; {m}], gbeta: &mut [i64; {m}],
        gw2: &mut [[i64; {m}]; {n}], gb2: &mut [i64; {n}],
        w1: &[[i64; {n}]; {m}], b1: &[i64; {m}], g: &[i64; {m}], beta: &[i64; {m}], w2: &[[i64; {m}]; {n}],
        b2: &[i64; {n}], x: &[[i64; {n}]; {b}], kind: &i64, eps: &i64) {{
    call mlp_layer_back_batch<{n}, {m}, {b}>(y, ay, ax, gw1, gb1, gg, gbeta, gw2, gb2, w1, b1, g, beta, w2, b2, x, kind, eps);
}}
"
    );
    lean_proves(
        "weave-batch-scale-mlp-layer",
        &src,
        &[
            "weave__mlp_layer_batch__mlp_layer_batch__64_64_32",
            "weave__mlp_layer_batch__mlp_layer_back_batch__64_64_32",
        ],
    );
}
