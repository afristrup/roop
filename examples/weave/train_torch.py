"""Ordinary float training in torch, with the semantics of weave's training step."""

import torch
import torch.nn.functional as F


def optimizer_for(params, kind, rate):
    """The torch optimizer that weave's `kind` stands for. The loss is summed over the
    batch, as weave sums gradients, so the same rate means the same step."""
    if kind == "sgd":
        return torch.optim.SGD(params, lr=rate)
    if kind == "momentum":
        return torch.optim.SGD(params, lr=rate, momentum=0.9)
    return torch.optim.Adam(params, lr=rate, eps=1e-3)


def fit(net, xs, ys, classes, epochs, rate, kind, batch):
    """Trains `net` on batches in the order given, no shuffling, as weave does.
    Returns, per epoch, the half squared error of the probabilities summed over
    the samples, which is what weave reports as its loss."""
    opt = optimizer_for(net.parameters(), kind, rate)
    targets = F.one_hot(ys, classes).double()
    curve = []
    for _ in range(epochs):
        total = 0.0
        for i in range(0, len(xs), batch):
            logits = net(xs[i : i + batch])
            loss = F.cross_entropy(logits, ys[i : i + batch], reduction="sum")
            opt.zero_grad()
            loss.backward()
            opt.step()
            probs = logits.detach().softmax(1)
            total += float(0.5 * ((probs - targets[i : i + batch]) ** 2).sum())
        curve.append(total)
    return curve
