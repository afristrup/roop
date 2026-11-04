# Weave and reversible learning

Weave is Roop's library for reversible neural networks. A usual training run
stores each layer input so that backpropagation can use it later. Weave rebuilds
the input by running each layer backwards. Its working memory therefore does
not increase with the number of layers.

A Weave layer changes a state made of two vectors, `q` and `p`. Its leapfrog
step has this form:

```text
p += -h/2 * W^T f(Wq + b)
q += h * p
p += -h/2 * W^T f(Wq + b)
```

Each update reads one part of the state and changes the other. The reverse run
subtracts the same values in reverse order. The force `f` need not itself have
an inverse.

Weave uses Q12 fixed-point arithmetic. An integer represents the real value
multiplied by 4096. That makes a completed forward and reverse layer return the
same bits. Floating-point code cannot make that promise.

## Compile a model

`roop weave` reads a model exported from a PyTorch `nn.Sequential` and writes a
Roop program. A project that uses it needs both `weave` and `einsum` modules.

```toml
[modules]
weave = "weave"
einsum = "einsum"
```

```text
roop weave model.json -o model.roop
roop test model.roop
roop lean model.roop --check
```

The generated file contains forward, backward, gradient, and training-step
functions. With batch output enabled it also contains versions that operate on
a fixed number of samples. Test output compares the fixed-point program with a
reference implementation in doubles.

Weave has perceptron, attention, convolution, and leapfrog layers. Its
attention block omits softmax because the block must add into a separate part of
the state to remain reversible. See [the language reference](../reference.md#weave)
for the layer definitions and [the memory measurement](../weave-memory.md) for
the measured storage cost.

[Previous: failure and rollback](06-rollback.md)

[Back to the contents](README.md)
