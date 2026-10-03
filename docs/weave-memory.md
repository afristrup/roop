# weave: memory against depth

weave trains a network of leapfrog layers without storing activations. The
backward pass runs the layers backward to get the activations back, so what it
needs does not grow with depth. This page measures that and says what it is worth.
Reproduce it with `python3 bench/weave_memory.py`.

## What is measured

One training step of `weave::step`, width 64, a batch of 8, on an Apple M4. Every
buffer the step uses is an argument: the weights and biases `ws`, `bs`, their
gradients `gw`, `gb`, and four vectors `q`, `p`, `aq`, `ap` of the layer width.
The table gives the peak resident memory of the whole process, and subtracts the
weights and gradients, which any implementation must hold:

| layers L | weights + gradients | peak resident | the rest |
|---:|---:|---:|---:|
| 8 | 0.5 MB | 6.4 MB | 5.82 MB |
| 32 | 2.1 MB | 8.5 MB | 6.37 MB |
| 128 | 8.5 MB | 14.9 MB | 6.36 MB |
| 512 | 34.1 MB | 40.5 MB | 6.37 MB |
| 1024 | 68.2 MB | 74.5 MB | 6.37 MB |

"The rest" is the same from 32 layers to 1024: a fixed 6.4 MB that is the program,
the runtime and the driver, and nothing that depends on depth. The step's own
working state is the four vectors, 2 KiB at this width.

## What a stored-activation pass would need

This part is a model, not a measurement: no stored-activation version of the same
network was built. A backward pass that keeps what it will read again holds, per
layer, `q`, `p`, and the three vectors the adjoint reads (`z`, `sigma(z)`,
`sigma'(z)`), five words of width per layer per sample in flight. Checkpointing every
`sqrt(L)` layers keeps `2 sqrt(L)` layers' worth and recomputes the rest, at the
price of extra forward work. For one sample in flight, width 64:

| layers L | roop | stored activations | checkpointed |
|---:|---:|---:|---:|
| 8 | 2.0 KiB | 22.0 KiB | 17.0 KiB |
| 32 | 2.0 KiB | 82.0 KiB | 32.0 KiB |
| 128 | 2.0 KiB | 322.0 KiB | 62.0 KiB |
| 512 | 2.0 KiB | 1282.0 KiB | 117.0 KiB |
| 1024 | 2.0 KiB | 2562.0 KiB | 162.0 KiB |

![working memory against depth](weave-memory.svg)

## What it is worth

- **The saving is real and grows with depth.** Constant against linear, and
  against square-root for checkpointing, with no recomputation of the forward
  pass: the backward pass is the inverse computation, so it costs about what the
  forward pass does.
- **At this width it is modest in absolute terms.** At 1024 layers the stored
  activations are 2.5 MiB for a sample, against 68 MB of weights and gradients,
  so the whole process would be 4% larger. The ratio of activations to weights is
  about `5 / N` per layer, so it matters when the network is narrow and deep, or
  when many samples are in flight at once, each of which multiplies the stored
  cost and the reversible one by the same factor.
- **Not claimed:** speed. Reversal is exact, but a training step here runs about
  750 samples a second at width 64 and 8 layers, and the layer type, the loss and
  the adjoints are written by hand. This shows that the memory claim holds. It is
  not a case that roop is a faster way to train a network than the usual tools.
