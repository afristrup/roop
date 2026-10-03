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
/// writes the trained weights.
pub fn emit_driver(model: &Model, batch: usize) -> String {
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
            "q",
            "p",
            "aq",
            "ap",
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
    let write: String = tensors
        .iter()
        .map(|t| format!("    fwrite({0}, 8, {1}, out);\n", t.name, t.data.len()))
        .collect();
    let types = vec!["int64_t*"; pointers.len()].join(", ");
    format!(
        r#"#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

{declare}static int64_t q[{n}], p[{n}], aq[{n}], ap[{n}];
static int64_t xs[{batch}][{n}], ts[{batch}][{k}];

void {name}_train({types});

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
        }}
        printf("%lld\n", (long long)total);
    }}
{write}    fclose(out);
    return 0;
}}
"#,
        args = pointers.join(", ")
    )
}
