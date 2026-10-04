"""The compiled reversible network as a torch module with trainable weights."""

import torch
from torch import nn

import env  # noqa: F401
from torch_mirror import build_layer, run_layers, snap


class MirrorNet(nn.Module):
    """The layers of an exported spec, run in doubles on a batch of states with the
    same function as the compiled code (activations, Pade tanh and all)."""

    def __init__(self, spec):
        super().__init__()
        self.spec = spec
        self.params = nn.ParameterList()
        self.layers = []
        for raw in spec["layers"]:
            layer, made = build_layer(raw, lambda v: nn.Parameter(snap(v)))
            self.params.extend(made)
            self.layers.append(layer)
        self.h = float(snap(spec["step"]))

    def one(self, x):
        return run_layers(self.layers, self.h, x)[: self.spec["outputs"]]

    def forward(self, xs):
        return torch.func.vmap(self.one)(xs)
