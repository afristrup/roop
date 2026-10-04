# weave on real tasks

Trains a reversible network on scikit-learn's bundled digits (8x8 images, 1797 samples) and
on next-character prediction over the text of `docs/reference.md`, and compares three ways of
training it. The results are in `docs/weave-real-models.md`.

| method | what it is |
| --- | --- |
| `plain` | an ordinary torch MLP of about the same size (second baseline) |
| `mirror` | the reversible network the compiler builds, as a torch module in doubles, trained by autograd (`mirror_net.py`, from `torch_mirror.py`) |
| `weave` | the same network, trained by `roop weave` in Q12 fixed point with no activations stored |

All three take batches in a fixed order with the loss summed over the batch, so a learning rate
means the same step to each. Torch's Adam has `eps=1e-3`, which is weave's 4/4096.

## Setup

```
cargo build --release -p roop -p roop-rt
cd crates/roop-weave/python && uv sync --extra torch
```

scikit-learn is in the `torch` extra. `env.py` finds `target/release/roop` (or `target/debug`);
set `WEAVE_ROOP` and `ROOP_RT_LIB` to use another build, and `WEAVE_LIBRARY` for another
copy of the `roop/` directory with the weave and einsum modules.

## Run

One configuration, one JSON line of results (curves, accuracy, loss, seconds, memory):

```
cd examples/weave
uv run --project ../../crates/roop-weave/python --extra torch python run.py digits \
    --method weave --optimizer adam --rate 0.001 --epochs 40 --depth 8 --seed 0
```

Options: `--task` is `digits` or `chars`; `--depth` is the number of layers (a leapfrog layer
and `depth - 1` blocks); `--kinds mlp,attention,conv` cycles the block kinds; `--hidden`,
`--act`, `--norm` (RMSNorm in the perceptrons), `--init-scale`, `--out-scale`, `--input-scale`,
`--batch`, `--cap` (fewer training samples), `--fixed-eval` (also test the compiled fixed point
forward pass, which is slow).

A list of configurations, one per line of arguments, with each run in its own process so that
peak memory is its own:

```
printf 'digits --method plain\ndigits --method weave\n' |
    uv run --project ../../crates/roop-weave/python --extra torch python sweep.py results.jsonl
python show.py results.jsonl method=weave
```

`results/` holds the runs behind the document, one `.jsonl` file per table.

## Test

```
uv run --project ../../crates/roop-weave/python --extra torch python -m unittest
```

`test_digits.py` trains the digits network for a few epochs and checks that the loss falls,
and `test_optimizers.py` checks weave's three optimizers against torch's on the same gradients.

## Memory and time

`weave` memory is the peak resident size of the compiled training program. For torch it is the
growth in peak RSS of the process, and the bytes autograd saves for the backward pass of one
batch (`saved`), which is the activation memory weave does not keep. Wall time is noisy on a
shared machine, so runs also record CPU seconds.
