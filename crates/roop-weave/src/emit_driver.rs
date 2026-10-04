use crate::Model;

/// A C program that trains the model that `emit_train` writes, on data in files:
///
/// ```text
/// prog <weights.bin> <data.bin> <epochs> <rate> <samples> <trained.bin>
/// ```
///
/// The files are little-endian 64-bit integers on the 1/4096 grid. The weights are
/// every tensor in the order of `Model::tensors`; the data is the inputs of all
/// the samples and then their targets. It prints the loss of each epoch, and
/// writes the trained weights. The arithmetic wraps, so a run that diverges would
/// report garbage; the program stops with status 3 when the loss or a weight
/// leaves `LIMIT` (2^31, a product of two such numbers still fits in 64 bits).
/// With `checked` it also stops with status 3 when the step set `roop_overflow`,
/// the flag of a program built with `overflow = true` under `[checks]`, which
/// catches a wrap deep inside a step that the loss and weights do not show.
pub fn emit_driver(model: &Model, batch: usize, checked: bool) -> String {
    let (n, k, name) = (model.width, model.outputs, &model.name);
    let tensors = model.tensors();
    let grads = model.gradients();
    let state = model.optimizer_state();
    let mut declare = String::new();
    let mut pointers = vec!["&total".to_string()];
    for t in tensors.iter().copied().chain(&grads).chain(&state) {
        declare += &format!("static int64_t {}[{}];\n", t.name, t.data.len());
        pointers.push(t.name.clone());
    }
    pointers.extend(
        [
            "(int64_t*)q",
            "(int64_t*)p",
            "(int64_t*)aq",
            "(int64_t*)ap",
            "(int64_t*)xs",
            "(int64_t*)ts",
            "&rate",
        ]
        .map(String::from),
    );
    let read: String = tensors
        .iter()
        .map(|t| format!("    load({0}, {1}, weights);\n", t.name, t.data.len()))
        .collect();
    let guard: String = tensors
        .iter()
        .map(|t| {
            format!(
                "            check(\"{0}\", {0}, {1});\n",
                t.name,
                t.data.len()
            )
        })
        .collect();
    let write: String = tensors
        .iter()
        .map(|t| format!("    fwrite({0}, 8, {1}, out);\n", t.name, t.data.len()))
        .collect();
    let (extern_flag, flag_check) = if checked {
        (
            "extern volatile int64_t roop_overflow;\n",
            "            if (roop_overflow) {\n                fprintf(stderr, \"overflow: an intermediate value wrapped inside a step; training diverged\\n\");\n                exit(3);\n            }\n",
        )
    } else {
        ("", "")
    };
    let types = vec!["int64_t*"; pointers.len()].join(", ");
    format!(
        r#"#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

{declare}static int64_t q[{batch}][{n}], p[{batch}][{n}], aq[{batch}][{n}], ap[{batch}][{n}];
static int64_t xs[{batch}][{n}], ts[{batch}][{k}];

void {name}_train({types});
{extern_flag}
#define LIMIT ((int64_t)1 << 31)

static void check(const char *what, const int64_t *v, size_t count) {{
    for (size_t i = 0; i < count; i++) {{
        if (v[i] >= LIMIT || v[i] <= -LIMIT) {{
            fprintf(stderr, "overflow: %s[%zu] is %lld, past +-%lld; training diverged\n", what, i, (long long)v[i], (long long)LIMIT);
            exit(3);
        }}
    }}
}}

static void load(int64_t *to, size_t count, FILE *from) {{
    if (fread(to, 8, count, from) != count) {{
        fprintf(stderr, "short read\n");
        exit(2);
    }}
}}

int main(int argc, char **argv) {{
    if (argc != 7) {{
        fprintf(stderr, "usage: %s weights data epochs rate samples trained\n", argv[0]);
        return 2;
    }}
    FILE *weights = fopen(argv[1], "rb"), *data = fopen(argv[2], "rb"), *out = fopen(argv[6], "wb");
    if (!weights || !data || !out) {{
        perror("open");
        return 2;
    }}
{read}    long epochs = atol(argv[3]), samples = atol(argv[5]);
    int64_t rate = atoll(argv[4]);
    if (samples % {batch} != 0) {{
        fprintf(stderr, "the samples must be a multiple of {batch}\n");
        return 2;
    }}
    int64_t *inputs = malloc(samples * {n} * 8), *targets = malloc(samples * {k} * 8);
    load(inputs, samples * {n}, data);
    load(targets, samples * {k}, data);
    for (long e = 0; e < epochs; e++) {{
        int64_t total = 0;
        for (long b = 0; b < samples; b += {batch}) {{
            for (int i = 0; i < {batch}; i++) {{
                for (int j = 0; j < {n}; j++) xs[i][j] = inputs[(b + i) * {n} + j];
                for (int j = 0; j < {k}; j++) ts[i][j] = targets[(b + i) * {k} + j];
            }}
            {name}_train({args});
{flag_check}            check("loss", &total, 1);
{guard}        }}
        printf("%lld\n", (long long)total);
    }}
{write}    fclose(out);
    return 0;
}}
"#,
        args = pointers.join(", ")
    )
}
