"""Trains a torch model with weave, and puts the trained weights back.

The model is exported, compiled by `roop weave`, built with the C program that
`roop weave --driver` writes, and trained there in fixed point with no
activations stored. The weights come back into the torch tensors they came from.
"""

import json
import os
import struct
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import torch

import contraction
from torch_to_weave import export_bound

GRID = 4096
LIBRARY = Path(__file__).resolve().parents[3] / "roop"


def pack(values):
    return struct.pack(f"<{len(values)}q", *[round(v * GRID) for v in values])


KEYS = {"leapfrog": ["weight", "bias"], "mlp": ["w1", "b1", "w2", "b2"],
        "residual": ["w1", "b1", "w2", "b2"],
        "attention": ["wq", "wk", "wv"], "conv": ["weight", "bias"]}


def tensors(spec):
    """Every tensor of the model, each as a flat list, in the order roop-weave keeps them."""
    out = []
    for layer in spec["layers"]:
        for key in KEYS[layer["kind"]]:
            value = layer[key]
            out.append(value if not isinstance(value[0], list) else [x for row in value for x in row])
        if layer.get("norm"):
            out.append(layer["norm"]["gain"])
            if layer["norm"].get("center"):
                out.append(layer["norm"]["bias"])
    return out


def run(command, usage=None):
    """The output of a command. `usage`, a dict, gets its wall `seconds`, its `cpu`
    seconds and its peak resident memory `rss` in bytes."""
    start = time.perf_counter()
    with tempfile.TemporaryFile("w+") as out, tempfile.TemporaryFile("w+") as err:
        process = subprocess.Popen(command, stdout=out, stderr=err, text=True)
        _, status, used = os.wait4(process.pid, 0)
        process.returncode = os.waitstatus_to_exitcode(status)
        out.seek(0)
        err.seek(0)
        text, errors = out.read(), err.read()
    if process.returncode != 0:
        raise RuntimeError(f"{' '.join(command)}\n{text}{errors}")
    if usage is not None:
        scale = 1 if sys.platform == "darwin" else 1024
        usage.update(seconds=time.perf_counter() - start, cpu=used.ru_utime + used.ru_stime,
                     rss=used.ru_maxrss * scale)
    return text


def project(spec, bound, target):
    """Scales the two weights of each residual block, in the torch tensors `bound`, so
    that its contraction bound is at most `target`. Biases are not touched."""
    at = 0
    for layer in spec["layers"]:
        count = len(KEYS[layer["kind"]])
        norm = layer.get("norm")
        count += 0 if not norm else 2 if norm.get("center") else 1
        if layer["kind"] == "residual":
            w1, w2 = bound[at], bound[at + 2]
            limit = contraction.bound(layer["activation"], w1.tolist(), w2.tolist())
            if limit > target:
                with torch.no_grad():
                    w1.mul_((target / limit) ** 0.5)
                    w2.mul_((target / limit) ** 0.5)
        at += count


def train(model, xs, ts, epochs, rate, outputs=None, loss="mse", optimizer=None,
          step=0.25, batch=None, roop="roop", name="net", library=LIBRARY, keep_contraction=None,
          every=50, usage=None, loss_scale=1):
    """Trains `model` in place on the samples xs (one row each) and their targets,
    and returns the loss of each epoch, summed over the samples. `library` is the
    directory with the weave and einsum modules, the `roop` of the repository. `usage`, a
    dict, gets the seconds and peak memory of the training program. `loss_scale` multiplies
    the seed of the backward pass, and the optimizer divides it out again.

    The contraction of a residual block is not kept by the training in weave: the
    update can take its weights past the bound, which leaves the block exactly
    invertible but makes the inverse of x + F(x) only approximate. With
    `keep_contraction` set to a bound below 0.9, training runs `every` epochs at a
    time, and after each the residual weights are scaled back to that bound (the state
    of momentum and Adam starts again at each stretch)."""
    xs, ts = torch.as_tensor(xs, dtype=torch.float64), torch.as_tensor(ts, dtype=torch.float64)
    outputs = ts.shape[1] if outputs is None else outputs
    options = (outputs, step, name, loss, optimizer or {"kind": "sgd"})
    spec, bound = export_bound(model.eval(), *options, loss_scale=loss_scale)
    chain = None
    if keep_contraction is not None:
        chain = contraction.iterations(keep_contraction)
        for layer in spec["layers"]:
            if layer["kind"] == "residual":
                layer["iters"] = chain
    batch = batch or min(len(xs), 32)
    stretch = epochs if keep_contraction is None else every
    losses = []
    with tempfile.TemporaryDirectory(prefix="weave-train-") as work:
        path = lambda f: os.path.join(work, f)
        with open(path("Roop.toml"), "w") as f:
            f.write(f'[modules]\nweave = "{library}/weave"\neinsum = "{library}/einsum"\n')
        with open(path("model.json"), "w") as f:
            json.dump(spec, f)
        run([roop, "weave", path("model.json"), "--batch", str(batch),
             "--driver", path("main.c"), "-o", path("prog.roop")])
        run([roop, "build", path("prog.roop"), "--link", path("main.c"), "-o", path("prog")])
        with open(path("data.bin"), "wb") as f:
            f.write(pack(xs.flatten().tolist()) + pack(ts.flatten().tolist()))
        for done in range(0, epochs, stretch):
            with open(path("weights.bin"), "wb") as f:
                f.write(pack([x for t in tensors(spec) for x in t]))
            used = {} if usage is not None else None
            out = run([path("prog"), path("weights.bin"), path("data.bin"),
                       str(min(stretch, epochs - done)), str(round(rate * GRID)), str(len(xs)),
                       path("trained.bin")], used)
            if usage is not None:
                for key in ("seconds", "cpu"):
                    usage[key] = usage.get(key, 0) + used[key]
                usage["rss"] = max(usage.get("rss", 0), used["rss"])
            losses += [int(line) / GRID for line in out.split()]
            with open(path("trained.bin"), "rb") as f:
                raw = f.read()
            put_back(bound, spec, struct.unpack(f"<{len(raw) // 8}q", raw))
            if keep_contraction is not None:
                project(spec, bound, keep_contraction)
                spec = export_bound(model.eval(), *options, loss_scale=loss_scale)[0]
                for layer in spec["layers"]:
                    if layer["kind"] == "residual":
                        layer["iters"] = chain
    return losses


def put_back(bound, spec, trained):
    """Copies the trained numbers into the torch tensors they came from."""
    at = 0
    with torch.no_grad():
        for tensor, listed in zip(bound, tensors(spec)):
            if tensor is not None:
                values = torch.tensor(trained[at:at + len(listed)], dtype=tensor.dtype) / GRID
                tensor.copy_(values.reshape(tensor.shape))
            at += len(listed)
