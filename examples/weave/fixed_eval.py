"""Test accuracy of a model in the fixed point arithmetic weave trains in: the model is
compiled with `roop weave --main` and run on each sample, so no float touches the forward
pass. The float mirror is what the other scripts evaluate with."""

import json
import os
import subprocess
import tempfile

import torch

import env
from torch_to_weave import export

GRID = 4096


def toolchain(spec, work, library):
    with open(os.path.join(work, "Roop.toml"), "w") as f:
        f.write(f'[modules]\nweave = "{library}/weave"\neinsum = "{library}/einsum"\nstd = "{library}/std"\n')
    with open(os.path.join(work, "model.json"), "w") as f:
        json.dump(spec, f)
    roop = env.roop()
    done = lambda args: subprocess.run(args, cwd=work, check=True, capture_output=True, text=True)
    done([roop, "weave", "model.json", "--main", "-o", "main.roop"])
    done([roop, "build", "main.roop", "-o", "main"])
    return os.path.join(work, "main")


def scores(program, x):
    args = [str(round(v * GRID)) for v in x.tolist()]
    out = subprocess.run([program, *args], capture_output=True, text=True, check=True).stdout
    return [int(line) / GRID for line in out.split()]


def fixed_accuracy(net, classes, xs, ys, library=None):
    """The accuracy of the torch model `net`, as weave would run it, on the samples."""
    spec = export(net.eval(), classes, loss="softmax")
    with tempfile.TemporaryDirectory(prefix="weave-eval-") as work:
        program = toolchain(spec, work, library or env.library())
        right = sum(int(torch.tensor(scores(program, x)).argmax() == y) for x, y in zip(xs, ys))
    return right / len(ys)
