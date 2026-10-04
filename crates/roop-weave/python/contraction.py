"""The contraction bound of a residual function, as roop-weave computes it.

A residual block x + F(x) is invertible by fixed-point iteration when F is a
contraction. For F(x) = W2 f(W1 x + b1) + b2 its Lipschitz constant is at most
||W2|| * L(f) * ||W1||, with ||W|| the spectral norm. The weights are first put on
the 1/4096 grid, since that is what weave holds, and the norm is a power iteration
taken 1 percent high, the same as in Rust.
"""

import math

LIMIT = 0.9
GRID = 4096

SLOPE = {"sigmoid": 0.2501, "silu": 1.1205, "gelu": 1.1205}


def snapped(rows):
    return [[math.floor(x * GRID + 0.5) / GRID for x in row] for row in rows]


def spectral_norm(rows):
    cols = len(rows[0])
    v = [1 / math.sqrt(cols)] * cols
    norm = 0.0
    for _ in range(500):
        wv = [sum(w * x for w, x in zip(row, v)) for row in rows]
        nxt = [sum(rows[r][c] * wv[r] for r in range(len(rows))) for c in range(cols)]
        norm = math.sqrt(sum(x * x for x in nxt))
        if norm == 0:
            return 0.0
        v = [x / norm for x in nxt]
    return math.sqrt(norm) * 1.01


def bound(activation, w1, w2):
    """An upper bound on the Lipschitz constant of W2 f(W1 x + b1) + b2."""
    slope = SLOPE.get(activation, 1.0)
    return spectral_norm(snapped(w1)) * spectral_norm(snapped(w2)) * slope


def iterations(bound_):
    """The cells of the fixed-point chain that bring its error below one unit of Q12."""
    steps = math.ceil(12 * math.log(2) / -math.log(bound_)) if bound_ > 0 else 0
    return max(steps, 3) + 1
