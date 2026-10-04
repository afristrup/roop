"""The datasets: digits from scikit-learn, and characters from the repository docs."""

import numpy as np
import torch
from sklearn.datasets import load_digits

import env

BATCH = 32


def split(x, y, test, cap=None):
    """Train and test parts, the train part a multiple of the batch (and at most `cap`)."""
    cut = int(len(y) * (1 - test)) // BATCH * BATCH
    if cap:
        cut = min(cut, cap // BATCH * BATCH)
    return x[:cut], y[:cut], x[len(y) - int(len(y) * test):], y[len(y) - int(len(y) * test):]


def digits(seed, test=0.25, cap=None):
    """Pixels scaled to 0..1, which fill a state of width 64, with the class of each,
    shuffled by `seed` and split in train and test."""
    data = load_digits()
    order = np.random.RandomState(seed).permutation(len(data.target))
    x = torch.tensor(data.data[order] / 16.0, dtype=torch.float64)
    return split(x, torch.tensor(data.target[order]), test, cap)


def characters(seed, test=0.2, cap=2048, context=4, vocabulary=16):
    """Next-character prediction on the text of docs/reference.md. The `context`
    characters are one-hot over the `vocabulary` most common (the rest share one slot),
    in a state of width context * vocabulary, which is 64."""
    text = (env.ROOT / "docs/reference.md").read_text().lower()
    common = sorted(set(text), key=lambda c: -text.count(c))[: vocabulary - 1]
    ids = [common.index(c) if c in common else vocabulary - 1 for c in text]
    rows = torch.tensor([ids[i : i + context + 1] for i in range(len(ids) - context)])
    rows = rows[torch.tensor(np.random.RandomState(seed).permutation(len(rows)))]
    x = torch.zeros(len(rows), context * vocabulary, dtype=torch.float64)
    for j in range(context):
        x[torch.arange(len(rows)), j * vocabulary + rows[:, j]] = 1.0
    return split(x, rows[:, context], test, cap)
