"""The torch architectures: a reversible stack weave accepts, and a plain MLP."""

import torch
from torch import nn

import env  # noqa: F401
from weave_modules import LinearAttention

ACTIVATIONS = {"tanh": nn.Tanh, "relu": nn.ReLU, "gelu": nn.GELU, "silu": nn.SiLU}


def block(kind, act, hidden, norm=False):
    """One reversible layer of the state of width 64 (8 rows of 8, or 8 channels of 8)."""
    if kind == "attention":
        return [LinearAttention(8, 8)]
    if kind == "conv":
        return [nn.Conv1d(8, 8, 3, padding=1)]
    middle = [nn.RMSNorm(hidden, eps=1e-2)] if norm else []
    return [nn.Linear(64, hidden), *middle, ACTIVATIONS[act](), nn.Linear(hidden, 64)]


def reversible(layers, kinds=("mlp",), hidden=32, act="tanh", out_scale=1.0, norm=False):
    """A leapfrog layer, which takes the input in, then `layers - 1` blocks cycling
    through `kinds` (mlp, attention, conv). The hidden width must not be 64, or the
    leapfrog layer and the first perceptron read as one perceptron. The last weights of
    each block are scaled by `out_scale`, since a deep stack adds a block to the state each
    layer and a state that grows with depth does not train. With `norm` the hidden
    layer of a perceptron is normalized by an RMSNorm."""
    parts = [nn.Linear(64, 64), ACTIVATIONS[act]()]
    for i in range(layers - 1):
        added = block(kinds[i % len(kinds)], act, hidden, norm)
        with torch.no_grad():
            for p in added[-1].parameters():
                p.mul_(out_scale)
        parts += added
    return nn.Sequential(*parts)


def plain(outputs, hidden=128, act="relu"):
    """An ordinary MLP of about the size of the reversible stack at depth 8."""
    a = ACTIVATIONS[act]
    return nn.Sequential(nn.Linear(64, hidden), a(), nn.Linear(hidden, hidden), a(), nn.Linear(hidden, outputs))
