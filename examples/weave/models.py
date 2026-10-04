"""The torch architectures: a reversible stack weave accepts, and a plain MLP."""

from torch import nn

import env  # noqa: F401
from weave_modules import LinearAttention

ACTIVATIONS = {"tanh": nn.Tanh, "relu": nn.ReLU, "gelu": nn.GELU, "silu": nn.SiLU}


def block(kind, act, hidden):
    """One reversible layer of the state of width 64 (8 rows of 8, or 8 channels of 8)."""
    if kind == "attention":
        return [LinearAttention(8, 8)]
    if kind == "conv":
        return [nn.Conv1d(8, 8, 3, padding=1)]
    return [nn.Linear(64, hidden), ACTIVATIONS[act](), nn.Linear(hidden, 64)]


def reversible(layers, kinds=("mlp",), hidden=32, act="tanh"):
    """A leapfrog layer, which takes the input in, then `layers - 1` blocks cycling
    through `kinds` (mlp, attention, conv). The hidden width must not be 64, or the
    leapfrog layer and the first perceptron read as one perceptron."""
    parts = [nn.Linear(64, 64), ACTIVATIONS[act]()]
    for i in range(layers - 1):
        parts += block(kinds[i % len(kinds)], act, hidden)
    return nn.Sequential(*parts)


def plain(outputs, hidden=128, act="relu"):
    """An ordinary MLP of about the size of the reversible stack at depth 8."""
    a = ACTIVATIONS[act]
    return nn.Sequential(nn.Linear(64, hidden), a(), nn.Linear(hidden, hidden), a(), nn.Linear(hidden, outputs))
