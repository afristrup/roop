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

from torch_to_weave import export_bound

GRID = 4096
LIBRARY = Path(__file__).resolve().parents[3] / "roop"


def pack(values):
    return struct.pack(f"<{len(values)}q", *[round(v * GRID) for v in values])


KEYS = {"leapfrog": ["weight", "bias"], "mlp": ["w1", "b1", "w2", "b2"],
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


def train(model, xs, ts, epochs, rate, outputs=None, loss="mse", optimizer=None,
          step=0.25, batch=None, roop="roop", name="net", library=LIBRARY, usage=None,
          loss_scale=1):
    """Trains `model` in place on the samples xs (one row each) and their targets,
    and returns the loss of each epoch, summed over the samples. `library` is the
    directory with the weave and einsum modules, the `roop` of the repository. `usage`, a
    dict, gets the seconds and peak memory of the training program. `loss_scale` multiplies
    the seed of the backward pass, and the optimizer divides it out again."""
    xs, ts = torch.as_tensor(xs, dtype=torch.float64), torch.as_tensor(ts, dtype=torch.float64)
    outputs = ts.shape[1] if outputs is None else outputs
    spec, bound = export_bound(model.eval(), outputs, step, name, loss, optimizer or {"kind": "sgd"},
                               loss_scale=loss_scale)
    batch = batch or min(len(xs), 32)
    with tempfile.TemporaryDirectory(prefix="weave-train-") as work:
        path = lambda f: os.path.join(work, f)
        with open(path("Roop.toml"), "w") as f:
            f.write(f'[modules]\nweave = "{library}/weave"\neinsum = "{library}/einsum"\n')
        with open(path("model.json"), "w") as f:
            json.dump(spec, f)
        run([roop, "weave", path("model.json"), "--batch", str(batch),
             "--driver", path("main.c"), "-o", path("prog.roop")])
        run([roop, "build", path("prog.roop"), "--link", path("main.c"), "-o", path("prog")])
        with open(path("weights.bin"), "wb") as f:
            f.write(pack([x for t in tensors(spec) for x in t]))
        with open(path("data.bin"), "wb") as f:
            f.write(pack(xs.flatten().tolist()) + pack(ts.flatten().tolist()))
        out = run([path("prog"), path("weights.bin"), path("data.bin"), str(epochs),
                   str(round(rate * GRID)), str(len(xs)), path("trained.bin")], usage)
        with open(path("trained.bin"), "rb") as f:
            raw = f.read()
    trained = struct.unpack(f"<{len(raw) // 8}q", raw)
    at = 0
    with torch.no_grad():
        for tensor, listed in zip(bound, tensors(spec)):
            if tensor is not None:
                values = torch.tensor(trained[at:at + len(listed)], dtype=tensor.dtype) / GRID
                tensor.copy_(values.reshape(tensor.shape))
            at += len(listed)
    return [int(line) / GRID for line in out.split()]
