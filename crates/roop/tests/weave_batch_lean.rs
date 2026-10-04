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

fn lean_accepts(name: &str, src: &str, names: &[&str]) {
    let dir = project(name, &config(), src);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in names {
        assert!(report.contains(name), "{name} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

#[test]
fn lean_proves_the_batched_forward_pass_exactly_reversible() {
    let src = "
use weave::layer_batch;
use weave::forward_batch;

fn one(q: &mut [[i64; 2]; 2], p: &mut [[i64; 2]; 2], w: &[[i64; 2]; 2], b: &[i64; 2], h: &i64, kind: &i64) {
    call layer_batch<2, 2, 2>(q, p, w, b, h, kind);
}
fn net(q: &mut [[i64; 2]; 2], p: &mut [[i64; 2]; 2], ws: &[[[i64; 2]; 2]; 2], bs: &[[i64; 2]; 2],
       h: &i64, kind: &i64) {
    call forward_batch<2, 2, 2, 2>(q, p, ws, bs, h, kind);
}
";
    lean_accepts(
        "weave-lean-batch",
        src,
        &["weave__net_batch__layer_batch__2_2_2", "one", "net"],
    );
}

#[test]
fn lean_proves_the_batched_backward_pass_exactly_reversible() {
    let src = "
use weave::layer_back_batch;
use weave::backward_batch;

fn one(q: &mut [[i64; 2]; 2], p: &mut [[i64; 2]; 2], aq: &mut [[i64; 2]; 2], ap: &mut [[i64; 2]; 2],
       gw: &mut [[i64; 2]; 2], gb: &mut [i64; 2], w: &[[i64; 2]; 2], b: &[i64; 2], h: &i64, kind: &i64) {
    call layer_back_batch<2, 2, 2>(q, p, aq, ap, gw, gb, w, b, h, kind);
}
fn net(q: &mut [[i64; 2]; 2], p: &mut [[i64; 2]; 2], aq: &mut [[i64; 2]; 2], ap: &mut [[i64; 2]; 2],
       gw: &mut [[[i64; 2]; 2]; 2], gb: &mut [[i64; 2]; 2],
       ws: &[[[i64; 2]; 2]; 2], bs: &[[i64; 2]; 2], h: &i64, kind: &i64) {
    call backward_batch<2, 2, 2, 2>(q, p, aq, ap, gw, gb, ws, bs, h, kind);
}
";
    lean_accepts(
        "weave-lean-batch-back",
        src,
        &[
            "weave__net_batch__force_back_batch__2_2_2",
            "weave__net_batch__layer_back_batch__2_2_2",
            "one",
            "net",
        ],
    );
}

#[test]
fn lean_proves_the_batched_perceptron_block_and_its_backward_step() {
    let src = "
use weave::mlp_batch;
use weave::mlp_back_batch;

fn fwd(y: &mut [[i64; 2]; 2], w1: &[[i64; 2]; 2], b1: &[i64; 2], w2: &[[i64; 2]; 2], b2: &[i64; 2],
       x: &[[i64; 2]; 2], kind: &i64) {
    call mlp_batch<2, 2, 2>(y, w1, b1, w2, b2, x, kind);
}
fn back(y: &mut [[i64; 2]; 2], ay: &[[i64; 2]; 2], ax: &mut [[i64; 2]; 2],
        gw1: &mut [[i64; 2]; 2], gb1: &mut [i64; 2], gw2: &mut [[i64; 2]; 2], gb2: &mut [i64; 2],
        w1: &[[i64; 2]; 2], b1: &[i64; 2], w2: &[[i64; 2]; 2], b2: &[i64; 2],
        x: &[[i64; 2]; 2], kind: &i64) {
    call mlp_back_batch<2, 2, 2>(y, ay, ax, gw1, gb1, gw2, gb2, w1, b1, w2, b2, x, kind);
}
";
    lean_accepts(
        "weave-lean-batch-mlp",
        src,
        &[
            "weave__mlp_batch__mlp_batch__2_2_2",
            "weave__mlp_batch__mlp_back_batch__2_2_2",
        ],
    );
}

#[test]
fn lean_proves_the_batched_attention_block_and_its_backward_step() {
    let src = "
use weave::attn_batch;
use weave::attn_back_batch;

fn fwd(y: &mut [[i64; 2]; 2], wq: &[[i64; 1]; 1], wk: &[[i64; 1]; 1], wv: &[[i64; 1]; 1], x: &[[i64; 2]; 2]) {
    call attn_batch<2, 1, 2, 2>(y, wq, wk, wv, x);
}
fn back(y: &mut [[i64; 2]; 2], ay: &[[i64; 2]; 2], ax: &mut [[i64; 2]; 2],
        gwq: &mut [[i64; 1]; 1], gwk: &mut [[i64; 1]; 1], gwv: &mut [[i64; 1]; 1],
        wq: &[[i64; 1]; 1], wk: &[[i64; 1]; 1], wv: &[[i64; 1]; 1], x: &[[i64; 2]; 2]) {
    call attn_back_batch<2, 1, 2, 2>(y, ay, ax, gwq, gwk, gwv, wq, wk, wv, x);
}
";
    lean_accepts(
        "weave-lean-batch-attn",
        src,
        &[
            "weave__attention_batch__attn_batch__2_1_2_2",
            "weave__attention_batch__attn_back_batch__2_1_2_2",
        ],
    );
}

#[test]
#[ignore = "about four minutes: run it with --ignored"]
fn lean_proves_the_batched_normalized_perceptron_and_its_backward_step() {
    let src = "
use weave::mlp_norm_batch;
use weave::mlp_norm_back_batch;

fn fwd(y: &mut [[i64; 2]; 2], w1: &[[i64; 2]; 2], b1: &[i64; 2], g: &[i64; 2], w2: &[[i64; 2]; 2],
       b2: &[i64; 2], x: &[[i64; 2]; 2], kind: &i64, eps: &i64) {
    call mlp_norm_batch<2, 2, 2>(y, w1, b1, g, w2, b2, x, kind, eps);
}
fn back(y: &mut [[i64; 2]; 2], ay: &[[i64; 2]; 2], ax: &mut [[i64; 2]; 2],
        gw1: &mut [[i64; 2]; 2], gb1: &mut [i64; 2], gg: &mut [i64; 2],
        gw2: &mut [[i64; 2]; 2], gb2: &mut [i64; 2],
        w1: &[[i64; 2]; 2], b1: &[i64; 2], g: &[i64; 2], w2: &[[i64; 2]; 2], b2: &[i64; 2],
        x: &[[i64; 2]; 2], kind: &i64, eps: &i64) {
    call mlp_norm_back_batch<2, 2, 2>(y, ay, ax, gw1, gb1, gg, gw2, gb2, w1, b1, g, w2, b2, x, kind, eps);
}
";
    lean_accepts(
        "weave-lean-batch-mlp-norm",
        src,
        &[
            "weave__mlp_norm_batch__mlp_norm_batch__2_2_2",
            "weave__mlp_norm_batch__mlp_norm_back_batch__2_2_2",
        ],
    );
}

#[test]
#[ignore = "minutes: run it with --ignored"]
fn lean_proves_the_batched_layer_normalized_perceptron_and_its_backward_step() {
    let src = "
use weave::mlp_layer_batch;
use weave::mlp_layer_back_batch;

fn fwd(y: &mut [[i64; 2]; 2], w1: &[[i64; 2]; 2], b1: &[i64; 2], g: &[i64; 2], beta: &[i64; 2],
       w2: &[[i64; 2]; 2], b2: &[i64; 2], x: &[[i64; 2]; 2], kind: &i64, eps: &i64) {
    call mlp_layer_batch<2, 2, 2>(y, w1, b1, g, beta, w2, b2, x, kind, eps);
}
fn back(y: &mut [[i64; 2]; 2], ay: &[[i64; 2]; 2], ax: &mut [[i64; 2]; 2],
        gw1: &mut [[i64; 2]; 2], gb1: &mut [i64; 2], gg: &mut [i64; 2], gbeta: &mut [i64; 2],
        gw2: &mut [[i64; 2]; 2], gb2: &mut [i64; 2],
        w1: &[[i64; 2]; 2], b1: &[i64; 2], g: &[i64; 2], beta: &[i64; 2], w2: &[[i64; 2]; 2],
        b2: &[i64; 2], x: &[[i64; 2]; 2], kind: &i64, eps: &i64) {
    call mlp_layer_back_batch<2, 2, 2>(y, ay, ax, gw1, gb1, gg, gbeta, gw2, gb2, w1, b1, g, beta, w2, b2, x, kind, eps);
}
";
    lean_accepts(
        "weave-lean-batch-mlp-layer",
        src,
        &[
            "weave__mlp_layer_batch__mlp_layer_batch__2_2_2",
            "weave__mlp_layer_batch__mlp_layer_back_batch__2_2_2",
        ],
    );
}
