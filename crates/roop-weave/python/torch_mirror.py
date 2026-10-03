"""The network roop-weave compiles, written in torch, for its autograd.

It runs the layers of a model exactly as the compiled code does, in doubles,
so the gradients torch finds can be compared with the reference of roop-weave.
"""

import torch

GRID = 4096.0


def snap(x):
    return torch.round(torch.as_tensor(x, dtype=torch.float64) * GRID) / GRID


def activate(name, z):
    if name == "identity":
        return z
    if name == "cauchy":
        return z / (1 + z * z)
    if name == "softsign":
        return z / (1 + z.abs())
    if name == "relu":
        return torch.clamp(z, min=0)
    if name == "tanh":
        pade = z * (27 + z * z) / (27 + 9 * z * z)
        return torch.where(z.abs() > 3, torch.sign(z), pade)
    raise ValueError(name)


def sample(width, outputs):
    x = [((i * 3 + 1) % 7) / 4 - 0.75 for i in range(width)]
    t = [((k * 2 + 1) % 3) / 4 - 0.25 for k in range(outputs)]
    return torch.tensor(x, dtype=torch.float64), torch.tensor(t, dtype=torch.float64)


def leaf(values):
    return snap(values).requires_grad_()


def leapfrog(layer, h, q, p):
    w, b, act = layer["w"], layer["b"], layer["activation"]

    def kick(q, p):
        return p - h / 2 * (w.T @ activate(act, w @ q + b))

    p = kick(q, p)
    q = q + h * p
    return q, kick(q, p)


def perceptron(layer, x):
    hidden = activate(layer["activation"], layer["w1"] @ x + layer["b1"])
    return layer["w2"] @ hidden + layer["b2"]


def mirror(spec):
    """The loss of the sample and the gradient of every tensor, in the order of
    the layers' tensors: w, b for a leapfrog layer and w1, b1, w2, b2 for a
    perceptron."""
    width, h = spec["width"], float(snap(spec["step"]))
    keys = {"leapfrog": [("w", "weight"), ("b", "bias")],
            "mlp": [("w1", "w1"), ("b1", "b1"), ("w2", "w2"), ("b2", "b2")]}
    layers, leaves = [], []
    for raw in spec["layers"]:
        layer = {"kind": raw["kind"], "activation": raw["activation"]}
        for ours, theirs in keys[raw["kind"]]:
            layer[ours] = leaf(raw[theirs])
            leaves.append(layer[ours])
        layers.append(layer)
    x, t = sample(width, spec["outputs"])
    x = snap(x)
    q, p = x, torch.zeros_like(x)
    into_q = True
    for layer in layers:
        if layer["kind"] == "leapfrog":
            q, p = leapfrog(layer, h, q, p)
        elif into_q:
            q = q + perceptron(layer, p)
            into_q = False
        else:
            p = p + perceptron(layer, q)
            into_q = True
    loss = 0.5 * ((q[: spec["outputs"]] - snap(t)) ** 2).sum()
    loss.backward()
    grads = [torch.zeros_like(l) if l.grad is None else l.grad for l in leaves]
    return float(loss), [g.flatten().tolist() for g in grads]
