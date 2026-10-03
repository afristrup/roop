"""Torch modules for the blocks of weave that torch has no module for."""

from torch import nn


class LinearAttention(nn.Module):
    """Attention without the softmax, over a state of `seq` rows of `dim` numbers:
    (Q K^T) V with Q, K and V the rows times a weight. weave has no exponential,
    so this is the attention it can run backward."""

    def __init__(self, seq, dim):
        super().__init__()
        self.seq, self.dim = seq, dim
        self.wq = nn.Linear(dim, dim, bias=False)
        self.wk = nn.Linear(dim, dim, bias=False)
        self.wv = nn.Linear(dim, dim, bias=False)

    def forward(self, x):
        rows = x.reshape(*x.shape[:-1], self.seq, self.dim)
        q, k, v = self.wq(rows), self.wk(rows), self.wv(rows)
        return ((q @ k.transpose(-1, -2)) @ v).reshape(x.shape)
