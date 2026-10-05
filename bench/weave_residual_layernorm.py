#!/usr/bin/env python3
"""A LayerNorm and residual network on sklearn digits, trained by weave and by torch.

    cd crates/roop-weave/python
    uv run --extra torch python ../../../bench/weave_residual_layernorm.py --seeds 0 1 2

The model is built in torch and trained twice from the same initial weights: in float by
torch's autograd and Adam, and exported to weave, which trains it in Q12 fixed point with
Adam, `keep_contraction` on the invertible residual blocks, and no stored activations.
Both runs take the loss summed over batches of 32 in the order of the data. One line
of the table per run, then the mean. `--checked` builds weave with `[checks] overflow = true`.
"""
import argparse
import json
import os
import pathlib
import resource
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "crates" / "roop-weave" / "python"))

import numpy as np
import torch
import torch.nn.functional as F
from sklearn.datasets import load_digits
from torch import nn

import contraction
from torch_mirror import build_layer, run_layers, snap
from torch_to_weave import export
from weave_train import train

BATCH = 32
KEPT = 0.8


class Residual(nn.Module):
    def __init__(self, hidden, scale):
        super().__init__()
        self.f = nn.Sequential(nn.Linear(64, hidden), nn.Tanh(), nn.Linear(hidden, 64))
        with torch.no_grad():
            for layer in (self.f[0], self.f[2]):
                layer.weight.mul_(scale)

    def forward(self, x):
        return x + self.f(x)


def model(blocks, hidden, seed):
    torch.manual_seed(seed)
    parts = [nn.Linear(64, 64), nn.Tanh()]
    for _ in range(blocks):
        parts += [nn.Linear(64, hidden), nn.LayerNorm(hidden, eps=1e-2), nn.GELU(), nn.Linear(hidden, 64),
                  Residual(hidden, 0.6)]
    net = nn.Sequential(*parts).double()
    with torch.no_grad():
        for m in net:
            if isinstance(m, nn.Linear) and m.out_features == 64 and m.in_features == hidden:
                m.weight.mul_(0.5)
    return net


class Mirror(nn.Module):
    """The function weave compiles for `net`, in doubles and trained by autograd. Each
    residual block runs the same fixed-point chain as weave's, of the length that the
    kept contraction needs."""

    def __init__(self, net):
        super().__init__()
        limit, contraction.LIMIT = contraction.LIMIT, float("inf")
        try:
            spec = export(net, 10, loss="softmax")
        finally:
            contraction.LIMIT = limit
        self.params, self.layers = nn.ParameterList(), []
        for raw in spec["layers"]:
            if raw["kind"] == "residual":
                raw["iters"] = contraction.iterations(KEPT)
            layer, made = build_layer(raw, lambda v: nn.Parameter(snap(v)))
            self.params.extend(made)
            self.layers.append(layer)
        self.h, self.outputs = float(snap(spec["step"])), spec["outputs"]

    def forward(self, xs):
        return torch.func.vmap(lambda x: run_layers(self.layers, self.h, x)[: self.outputs])(xs)


def digits(seed):
    data = load_digits()
    order = np.random.RandomState(seed).permutation(len(data.target))
    x = torch.tensor(data.data[order] / 16.0, dtype=torch.float64)
    y = torch.tensor(data.target[order])
    cut = int(len(y) * 0.75) // BATCH * BATCH
    return x[:cut], y[:cut], x[-449:], y[-449:]


def metrics(net, x, y):
    with torch.no_grad():
        logits = net(x)
    return {"acc": float((logits.argmax(1) == y).double().mean()), "ce": float(F.cross_entropy(logits, y))}


def float_train(net, xs, ys, epochs, rate):
    opt = torch.optim.Adam(net.parameters(), lr=rate, eps=1e-3)
    onehot = F.one_hot(ys, 10).double()
    curve = []
    for _ in range(epochs):
        total = 0.0
        for i in range(0, len(xs), BATCH):
            logits = net(xs[i:i + BATCH])
            loss = F.cross_entropy(logits, ys[i:i + BATCH], reduction="sum")
            opt.zero_grad()
            loss.backward()
            opt.step()
            total += float(0.5 * ((logits.detach().softmax(1) - onehot[i:i + BATCH]) ** 2).sum())
        curve.append(round(total / len(xs), 5))
    return curve


def residual_bound(mirror):
    return max(contraction.bound(l["activation"], l["w1"].tolist(), l["w2"].tolist())
               for l in mirror.layers if l["kind"] == "residual")


def one(args):
    xs, ys, xt, yt = digits(args.seed)
    net = model(args.blocks, args.hidden, args.seed)
    out = {"method": args.method, "seed": args.seed}
    start = time.perf_counter()
    if args.method == "weave":
        usage = {}
        targets = F.one_hot(ys, 10).double()
        losses = train(net, xs, targets, args.epochs, args.rate, loss="softmax", optimizer={"kind": "adam"},
                       roop=os.environ.get("WEAVE_ROOP", "roop"), keep_contraction=KEPT, usage=usage,
                       checked=args.checked)
        out.update(usage)
        out["curve"] = [round(l / len(ys), 5) for l in losses]
        net = Mirror(net)
    else:
        net = Mirror(net)
        base = resource.getrusage(resource.RUSAGE_SELF)
        out["curve"] = float_train(net, xs, ys, args.epochs, args.rate)
        used = resource.getrusage(resource.RUSAGE_SELF)
        out["cpu"] = used.ru_utime + used.ru_stime - base.ru_utime - base.ru_stime
        out["rss"] = used.ru_maxrss
    out["seconds"] = time.perf_counter() - start
    out["test"], out["train"] = metrics(net, xt, yt), metrics(net, xs, ys)
    out["bound"] = residual_bound(net)
    print(json.dumps(out))


def run_one(args, method, seed):
    cmd = [sys.executable, __file__, "--one", "--method", method, "--seed", str(seed)] + [
        f"--{k}={getattr(args, k)}" for k in ("epochs", "rate", "blocks", "hidden")
    ] + (["--checked"] if args.checked else [])
    done = subprocess.run(cmd, capture_output=True, text=True)
    if done.returncode != 0:
        print(f"seed {seed} {method} failed:\n{done.stdout}{done.stderr}", file=sys.stderr)
        return {"method": method, "seed": seed, "failed": True}
    return json.loads(done.stdout.strip().splitlines()[-1])


def show(r):
    if r.get("failed"):
        return f"| {r['method']} | {r['seed']} | failed | | | | | | |"
    c = r["curve"]
    return (f"| {r['method']} | {r['seed']} | {r['test']['acc']:.3f} | {r['test']['ce']:.3f} | {r['train']['ce']:.3f} "
            f"| {c[0]:.3f} / {c[min(9, len(c) - 1)]:.3f} / {c[-1]:.4f} | {r['bound']:.3f} | {r['cpu']:.1f} "
            f"| {r['rss'] / 1e6:.0f} |")


def sweep(args):
    rows = [run_one(args, method, seed) for seed in args.seeds for method in ("torch", "weave")]
    print("| method | seed | test acc | test CE | train CE | loss ep1 / ep10 / last | residual bound | cpu s | RSS MB |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for r in rows:
        print(show(r))
    for method in ("torch", "weave"):
        ok = [r for r in rows if r["method"] == method and not r.get("failed")]
        if ok:
            a = np.array([r["test"]["acc"] for r in ok])
            print(f"\n{method}: test acc {a.mean():.3f} +- {a.std():.3f} over {len(ok)} seeds")


def main():
    a = argparse.ArgumentParser()
    a.add_argument("--seeds", type=int, nargs="+", default=[0, 1, 2])
    a.add_argument("--seed", type=int, default=0)
    a.add_argument("--one", action="store_true")
    a.add_argument("--method", choices=["torch", "weave"], default="weave")
    a.add_argument("--epochs", type=int, default=30)
    a.add_argument("--rate", type=float, default=0.001)
    a.add_argument("--blocks", type=int, default=4)
    a.add_argument("--hidden", type=int, default=32)
    a.add_argument("--checked", action="store_true")
    args = a.parse_args()
    one(args) if args.one else sweep(args)


if __name__ == "__main__":
    main()
