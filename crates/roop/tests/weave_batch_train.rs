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

fn grad_source(n: usize, m: usize, l: usize, k: usize, b: usize) -> String {
    format!(
        "
use weave::grad_batch;

fn grad(total: &mut i64, q: &mut [[i64; {n}]; {b}], p: &mut [[i64; {n}]; {b}],
        aq: &mut [[i64; {n}]; {b}], ap: &mut [[i64; {n}]; {b}],
        gw: &mut [[[i64; {n}]; {m}]; {l}], gb: &mut [[i64; {m}]; {l}],
        ws: &[[[i64; {n}]; {m}]; {l}], bs: &[[i64; {m}]; {l}],
        h: &i64, kind: &i64, t: &[[i64; {k}]; {b}]) {{
    call grad_batch<{n}, {m}, {l}, {k}, {b}>(total, q, p, aq, ap, gw, gb, ws, bs, h, kind, t);
}}
"
    )
}

fn attention_source(s: usize, d: usize, b: usize) -> String {
    let n = s * d;
    format!(
        "
use weave::attn_batch;
use weave::attn_back_batch;

fn fwd(y: &mut [[i64; {n}]; {b}], wq: &[[i64; {d}]; {d}], wk: &[[i64; {d}]; {d}], wv: &[[i64; {d}]; {d}],
       x: &[[i64; {n}]; {b}]) {{
    call attn_batch<{s}, {d}, {n}, {b}>(y, wq, wk, wv, x);
}}
fn back(y: &mut [[i64; {n}]; {b}], ay: &[[i64; {n}]; {b}], ax: &mut [[i64; {n}]; {b}],
        gwq: &mut [[i64; {d}]; {d}], gwk: &mut [[i64; {d}]; {d}], gwv: &mut [[i64; {d}]; {d}],
        wq: &[[i64; {d}]; {d}], wk: &[[i64; {d}]; {d}], wv: &[[i64; {d}]; {d}], x: &[[i64; {n}]; {b}]) {{
    call attn_back_batch<{s}, {d}, {n}, {b}>(y, ay, ax, gwq, gwk, gwv, wq, wk, wv, x);
}}
"
    )
}

#[test]
fn lean_proves_the_batched_attention_block_at_4_rows_4_tokens_and_width_4() {
    lean_proves(
        "weave-attn-batch-4",
        &attention_source(4, 4, 4),
        &[
            "weave__attention_batch__attn_batch__4_4_16_4",
            "weave__attention_batch__attn_back_batch__4_4_16_4",
        ],
    );
}

#[test]
fn lean_proves_the_batched_gradient_at_32_rows_width_64_and_3_layers() {
    lean_proves(
        "weave-grad-batch-scale",
        &grad_source(64, 64, 3, 8, 32),
        &["weave__train_batch__grad_batch__64_64_3_8_32"],
    );
}

#[test]
fn lean_proves_the_batched_gradient_at_2_rows_width_2_and_2_layers() {
    lean_proves(
        "weave-grad-batch-small",
        &grad_source(2, 2, 2, 2, 2),
        &["weave__train_batch__grad_batch__2_2_2_2_2"],
    );
}
