"""Reads a torch model and writes the JSON that roop-weave compiles.

weave layers are reversible, and most torch layers are not, so the model is
read as a network of reversible blocks, found in an nn.Sequential:

    Linear(N, M), act, Linear(M, N)   an "mlp" block: one half of the state takes
                                      in a perceptron of the other (RevNet, NICE)
    LinearAttention(seq, dim)         an "attention" block, a half of a coupling like
                                      the perceptron, from weave_modules.py
    Linear(N, M), act                 a "leapfrog" layer: a step of the dynamics
                                      whose force is W^T act(W q + b), with the
                                      weight tied to its transpose

The activations are Identity, ReLU, Tanh, Softsign, Sigmoid, SiLU and GELU
(GELU as the usual z * sigmoid(1.702 z) approximation). Everything else is
refused, naming the layer, rather than turned into something that is not what
the model computes. The compiled network is the network of these blocks, run
on a state (q, p) with the input in q and p zero; it is not the original
module run unchanged, and the Rust side tests it against its own reference.

    python3 torch_to_weave.py pkg.module:factory --outputs 1 -o model.json
"""

import argparse
import importlib
import json
import sys

ACTIVATIONS = {
    "Identity": "identity",
    "ReLU": "relu",
    "Tanh": "tanh",
    "Softsign": "softsign",
    "Sigmoid": "sigmoid",
    "SiLU": "silu",
    "GELU": "gelu",
}


class Unsupported(Exception):
    """The model has a layer weave cannot run backward."""


def flatten(module):
    """The leaf modules of a model, in order."""
    children = list(module.children())
    if not children or kind(module) == "LinearAttention":
        return [module]
    return [leaf for child in children for leaf in flatten(child)]


def activation_named(name):
    """A stand-in for an activation module, for one called as a function."""
    return type(name, (), {"children": lambda self: iter(())})()


FUNCTIONS = {
    "relu": "ReLU",
    "tanh": "Tanh",
    "softsign": "Softsign",
    "sigmoid": "Sigmoid",
    "silu": "SiLU",
    "gelu": "GELU",
}


def traced_chain(model):
    """The layers of a model that is not an nn.Sequential, in order, found by
    tracing it. The graph must be a chain: each layer takes the one before."""
    import torch.fx

    class Tracer(torch.fx.Tracer):
        def is_leaf_module(self, module, name):
            return kind(module) == "LinearAttention" or super().is_leaf_module(module, name)

    modules = dict(model.named_modules())
    leaves, previous = [], None
    for node in Tracer().trace(model).nodes:
        if node.op == "placeholder":
            previous = node
        elif node.op == "output":
            if node.args[0] is not previous:
                raise Unsupported("the output is not the last layer")
        elif node.args[:1] != (previous,) or len(node.all_input_nodes) != 1:
            raise Unsupported(f"{node.name}: the graph branches, and weave runs a chain")
        elif node.op == "call_module":
            leaves.append(modules[node.target])
            previous = node
        elif node.op == "call_function" and getattr(node.target, "__name__", "") in FUNCTIONS:
            leaves.append(activation_named(FUNCTIONS[node.target.__name__]))
            previous = node
        else:
            raise Unsupported(f"{node.name}: {node.target} is not a layer weave can run backward")
    return leaves


def kind(module):
    return type(module).__name__


def is_linear(module):
    return kind(module) == "Linear"


def numbers(tensor):
    return tensor.detach().cpu().tolist()


def bias_of(linear):
    if linear.bias is None:
        return [0.0] * linear.out_features
    return numbers(linear.bias)


def activation_of(module, position):
    name = ACTIVATIONS.get(kind(module))
    if name is None:
        raise Unsupported(
            f"layer {position}: {kind(module)} is not reversible; use one of "
            + ", ".join(sorted(ACTIVATIONS))
        )
    return name


def attention_block(module, i, width):
    if module.seq * module.dim != width:
        raise Unsupported(
            f"layer {i}: LinearAttention reads {module.seq * module.dim} numbers, "
            f"but the state is {width} wide"
        )
    return {
        "kind": "attention",
        "seq": module.seq,
        "wq": numbers(module.wq.weight),
        "wk": numbers(module.wk.weight),
        "wv": numbers(module.wv.weight),
    }


def block_at(leaves, i, width):
    """The block that starts at leaves[i], and how many leaves it takes."""
    first = leaves[i]
    if kind(first) == "LinearAttention":
        return attention_block(first, i, width), 1
    if not is_linear(first):
        raise Unsupported(f"layer {i}: {kind(first)} must follow a Linear")
    if first.in_features != width:
        raise Unsupported(
            f"layer {i}: Linear takes {first.in_features} numbers, "
            f"but the state is {width} wide"
        )
    if i + 1 >= len(leaves):
        raise Unsupported(f"layer {i}: a Linear with no activation is not reversible")
    act = activation_of(leaves[i + 1], i + 1)
    hidden = first.out_features
    closes = i + 2 < len(leaves) and is_linear(leaves[i + 2])
    if closes and leaves[i + 2].in_features == hidden and leaves[i + 2].out_features == width:
        second = leaves[i + 2]
        block = {
            "kind": "mlp",
            "activation": act,
            "w1": numbers(first.weight),
            "b1": bias_of(first),
            "w2": numbers(second.weight),
            "b2": bias_of(second),
        }
        return block, 3
    block = {
        "kind": "leapfrog",
        "activation": act,
        "weight": numbers(first.weight),
        "bias": bias_of(first),
    }
    return block, 2


def parameters(block, leaves, i):
    """The torch tensors of a block, in the order of its tensors in roop-weave. A
    bias that does not exist is None."""
    if block["kind"] == "attention":
        m = leaves[i]
        return [m.wq.weight, m.wk.weight, m.wv.weight]
    first = leaves[i]
    if block["kind"] == "leapfrog":
        return [first.weight, first.bias]
    second = leaves[i + 2]
    return [first.weight, first.bias, second.weight, second.bias]


def export_bound(model, outputs=None, step=0.25, name="model", loss="mse", optimizer=None):
    """The roop-weave model for a torch module as a dict, and the torch tensors
    that hold each of its weights, in the order roop-weave keeps them."""
    leaves = flatten(model) if kind(model) == "Sequential" else traced_chain(model)
    heads = [m for m in leaves if is_linear(m) or kind(m) == "LinearAttention"]
    if not heads:
        raise Unsupported("the model has no Linear layer")
    first = heads[0]
    width = first.seq * first.dim if kind(first) == "LinearAttention" else first.in_features
    layers, bound, i = [], [], 0
    while i < len(leaves):
        block, taken = block_at(leaves, i, width)
        layers.append(block)
        bound.extend(parameters(block, leaves, i))
        i += taken
    spec = {
        "name": name,
        "width": width,
        "outputs": width if outputs is None else outputs,
        "step": step,
        "layers": layers,
    }
    if loss != "mse":
        spec["loss"] = loss
    if optimizer is not None:
        spec["optimizer"] = optimizer
    return spec, bound


def export(model, outputs=None, step=0.25, name="model", loss="mse", optimizer=None):
    """The roop-weave model for a torch module, as a dict."""
    return export_bound(model, outputs, step, name, loss, optimizer)[0]


def load(spec):
    module_name, _, factory = spec.partition(":")
    sys.path.insert(0, ".")
    return getattr(importlib.import_module(module_name), factory)()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("model", help="module:factory, which returns the nn.Module")
    parser.add_argument("--outputs", type=int, help="how many outputs the loss reads")
    parser.add_argument("--step", type=float, default=0.25, help="leapfrog step size")
    parser.add_argument("--name", default="model", help="prefix of the roop functions")
    parser.add_argument("--loss", default="mse", choices=["mse", "sigmoid", "softmax"])
    parser.add_argument("--optimizer", default="sgd", choices=["sgd", "momentum", "adam"])
    parser.add_argument("-o", "--output", required=True)
    args = parser.parse_args(argv)
    try:
        optimizer = {"kind": args.optimizer}
        net = load(args.model).eval()
        spec = export(net, args.outputs, args.step, args.name, args.loss, optimizer)
    except Unsupported as why:
        sys.exit(f"torch_to_weave: {why}")
    with open(args.output, "w") as out:
        json.dump(spec, out)


if __name__ == "__main__":
    main()
