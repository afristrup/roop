"""The network roop-weave compiles, written in torch, for its autograd.

It runs the layers of a model exactly as the compiled code does, in doubles,
so the gradients torch finds can be compared with the reference of roop-weave.
"""

import torch

import contraction

GRID = 4096.0

KEYS = {"leapfrog": [("w", "weight"), ("b", "bias")],
        "mlp": [("w1", "w1"), ("b1", "b1"), ("w2", "w2"), ("b2", "b2")],
        "residual": [("w1", "w1"), ("b1", "b1"), ("w2", "w2"), ("b2", "b2")],
        "attention": [("wq", "wq"), ("wk", "wk"), ("wv", "wv")],
        "conv": [("w", "weight"), ("b", "bias")]}


def snap(x):
    return torch.round(torch.as_tensor(x, dtype=torch.float64) * GRID) / GRID


GELU_SCALE = 6971 / 4096


def pade(z):
    return torch.where(z.abs() > 3, torch.sign(z), z * (27 + z * z) / (27 + 9 * z * z))


def sigmoid(z):
    return 0.5 + 0.5 * pade(z / 2)


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
        return pade(z)
    if name == "sigmoid":
        return sigmoid(z)
    if name == "silu":
        return z * sigmoid(z)
    if name == "gelu":
        return z * sigmoid(GELU_SCALE * z)
    raise ValueError(name)


def sample(width, outputs, loss="mse"):
    x = [((i * 3 + 1) % 7) / 4 - 0.75 for i in range(width)]
    t = {
        "mse": [((k * 2 + 1) % 3) / 4 - 0.25 for k in range(outputs)],
        "sigmoid": [float(k % 2) for k in range(outputs)],
        "softmax": [float(k == 0) for k in range(outputs)],
    }[loss]
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
    z = layer["w1"] @ x + layer["b1"]
    if layer.get("g") is not None:
        if layer.get("beta") is not None:
            z = z - z.mean()
        z = z * torch.rsqrt((z * z).mean() + layer["eps"]) * layer["g"]
        if layer.get("beta") is not None:
            z = z + layer["beta"]
    hidden = activate(layer["activation"], z)
    return layer["w2"] @ hidden + layer["b2"]


def residual(layer, q, p):
    """The four lines of the residual block of weave: p takes in x + F(x), q is cleared
    of x by the fixed-point iteration x_(k+1) = y - F(x_k) run `iters - 1` times from
    y, then q and p are rotated so that q is y and p is minus the remnant."""
    p = p + q + perceptron(layer, q)
    x = p
    for _ in range(layer["iters"] - 1):
        x = p - perceptron(layer, x)
    q = q - x
    q = q + p
    return q, p - q


def convolve(layer, x):
    channels, kernel = layer["channels"], layer["kernel"]
    rows = x.reshape(1, channels, -1)
    out = torch.nn.functional.conv1d(
        rows, layer["w"].reshape(channels, channels, kernel), layer["b"], padding=kernel // 2
    )
    return out.reshape(-1)


def attention(layer, x):
    seq = layer["seq"]
    rows = x.reshape(seq, -1)
    q, k, v = rows @ layer["wq"].T, rows @ layer["wk"].T, rows @ layer["wv"].T
    return ((q @ k.T) @ v).reshape(-1)


HALVES = {"attention": lambda layer, x: attention(layer, x),
          "conv": lambda layer, x: convolve(layer, x)}


def iterations(raw):
    """The chain length roop-weave takes for a residual block that gives none."""
    return contraction.iterations(contraction.bound(raw["activation"], raw["w1"], raw["w2"]))


def build_layer(raw, make):
    """The layer of a spec with each tensor made by `make`, and those tensors in the
    order roop-weave keeps them."""
    layer = {"kind": raw["kind"], "activation": raw.get("activation"), "seq": raw.get("seq"),
             "channels": raw.get("channels"), "kernel": raw.get("kernel")}
    made = []
    for ours, theirs in KEYS[raw["kind"]]:
        layer[ours] = make(raw[theirs])
        made.append(layer[ours])
    if raw.get("norm"):
        layer["g"], layer["eps"] = make(raw["norm"]["gain"]), raw["norm"]["eps"]
        made.append(layer["g"])
        if raw["norm"].get("center"):
            layer["beta"] = make(raw["norm"]["bias"])
            made.append(layer["beta"])
    if raw["kind"] == "residual":
        layer["iters"] = raw.get("iters") or iterations(raw)
    return layer, made


def run_layers(layers, h, x):
    q, p, into_q = x, torch.zeros_like(x), True
    for layer in layers:
        if layer["kind"] == "leapfrog":
            q, p = leapfrog(layer, h, q, p)
        elif layer["kind"] == "residual":
            q, p = residual(layer, q, p)
        else:
            half = HALVES.get(layer["kind"], perceptron)
            if into_q:
                q = q + half(layer, p)
            else:
                p = p + half(layer, q)
            into_q = not into_q
    return q


def evaluate(spec, xs, ts):
    """The loss, summed over the samples, of the model as it is, in doubles. This
    is what the training of weave reports, in fixed point."""
    layers = [build_layer(raw, snap)[0] for raw in spec["layers"]]
    h, total = float(snap(spec["step"])), 0.0
    for x, t in zip(xs, ts):
        q = run_layers(layers, h, snap(x))[: spec["outputs"]]
        total += float(0.5 * ((q - snap(t)) ** 2).sum())
    return total


def mirror(spec):
    """The loss of the sample and the gradient of every tensor, in the order of
    the layers' tensors: w, b for a leapfrog layer and w1, b1, w2, b2 for a
    perceptron or a residual block, then the gain and the bias of its norm."""
    width, h = spec["width"], float(snap(spec["step"]))
    layers, leaves = [], []
    for raw in spec["layers"]:
        layer, made = build_layer(raw, leaf)
        layers.append(layer)
        leaves.extend(made)
    loss_kind = spec.get("loss", "mse")
    x, t = sample(width, spec["outputs"], loss_kind)
    x = snap(x)
    q = run_layers(layers, h, x)
    out, target = q[: spec["outputs"]], snap(t)
    if loss_kind == "sigmoid":
        loss = torch.nn.functional.binary_cross_entropy_with_logits(out, target, reduction="sum")
    elif loss_kind == "softmax":
        loss = -(target * torch.log_softmax(out, dim=0)).sum()
    else:
        loss = 0.5 * ((out - target) ** 2).sum()
    loss.backward()
    grads = [torch.zeros_like(l) if l.grad is None else l.grad for l in leaves]
    return loss.item(), [g.flatten().tolist() for g in grads]
