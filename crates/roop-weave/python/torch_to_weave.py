"""Reads a torch model and writes the JSON that roop-weave compiles.

weave layers are reversible, and most torch layers are not, so the model is
read as a network of reversible blocks, found in an nn.Sequential:

    Linear(N, M), RMSNorm(M), act, Linear(M, N)
                                      the same with the hidden layer normalized
    Linear(N, M), LayerNorm(M), act, Linear(M, N)
                                      the same with a layer norm: centered, scaled,
                                      shifted by its bias (a norm with "center")
    x + F(x), F a Linear, act, Linear  a "residual" block, an invertible residual
                                      connection (see below)
    Linear(N, M), act, Linear(M, N)   an "mlp" block: one half of the state takes
                                      in a perceptron of the other (RevNet, NICE)
    Conv1d(C, C, K, padding=K // 2)   a "conv" block, a half of a coupling like the
                                      perceptron; pass width= when no Linear comes first
    LinearAttention(seq, dim)         an "attention" block, a half of a coupling like
                                      the perceptron, from weave_modules.py
    Linear(N, M), act                 a "leapfrog" layer: a step of the dynamics
                                      whose force is W^T act(W q + b), with the
                                      weight tied to its transpose

A residual connection is x + F(x) with F one Linear, activation, Linear block, found
by tracing (an nn.Sequential holding such a module is traced too). It is refused
unless F is a contraction: the spectral norms of its weights and the slope of the
activation must multiply to less than 0.9, since weave inverts the block by
fixed-point iteration, which only converges then. Anything else that adds two
paths is refused.

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
import operator
import sys

import contraction

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


class Residual:
    """The leaves of a chain F whose output is added to its input, x + F(x)."""

    def __init__(self, leaves):
        self.leaves = leaves

    def children(self):
        return iter(())


def is_plain(module):
    """Whether a model is made of nn.Sequential and leaf modules only, so that
    flattening it loses nothing. A module of its own may add paths, and is traced."""
    for child in module.children():
        holds_modules = any(True for _ in child.children())
        if holds_modules and kind(child) != "LinearAttention":
            if kind(child) != "Sequential" or not is_plain(child):
                return False
    return True


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
    """The layers of a model that is not a plain nn.Sequential, in order, found by
    tracing it. The graph must be a chain, except that a chain may be added to its
    own input: x + F(x) becomes a Residual of the layers of F."""
    import torch
    import torch.fx

    class Tracer(torch.fx.Tracer):
        def is_leaf_module(self, module, name):
            return kind(module) == "LinearAttention" or super().is_leaf_module(module, name)

    modules = dict(model.named_modules())
    leaves, previous, depth = [], None, {}
    for node in Tracer().trace(model).nodes:
        if node.op == "placeholder":
            previous = node
        elif node.op == "output":
            if node.args[0] is not previous:
                raise Unsupported("the output is not the last layer")
        elif is_add(node, torch):
            origin = [n for n in node.all_input_nodes if n is not previous]
            if len(node.all_input_nodes) != 2 or len(origin) != 1 or origin[0] not in depth:
                raise Unsupported(
                    f"{node.name}: weave adds only a chain to its own input, x + F(x)"
                )
            start = depth[origin[0]]
            if start == len(leaves):
                raise Unsupported(f"{node.name}: x + x has no function to invert")
            leaves[start:] = [Residual(leaves[start:])]
            previous = node
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
        depth[previous] = len(leaves)
    return leaves


def is_add(node, torch):
    adds = node.op == "call_function" and node.target in (operator.add, torch.add)
    return adds or (node.op == "call_method" and node.target == "add")


def kind(module):
    return type(module).__name__


def is_linear(module):
    return kind(module) == "Linear"


def numbers(tensor):
    return tensor.detach().cpu().tolist()


def bias_of(linear):
    if linear.bias is None:
        return [0.0] * (linear.out_channels if kind(linear) == "Conv1d" else linear.out_features)
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


NORMS = ("RMSNorm", "LayerNorm")


def norm_of(module, i, hidden):
    name = kind(module)
    eps = module.eps if module.eps is not None else 1.1920929e-07
    if eps * 4096 < 1:
        raise Unsupported(
            f"layer {i}: {name} eps {eps} is below 1/4096, the smallest weave holds; give it eps=1e-3"
        )
    if tuple(module.normalized_shape) != (hidden,):
        raise Unsupported(f"layer {i}: {name} must normalize the {hidden} hidden units")
    weight = module.weight
    norm = {"eps": eps, "gain": [1.0] * hidden if weight is None else numbers(weight)}
    if name == "LayerNorm":
        bias = module.bias
        norm["center"] = True
        norm["bias"] = [0.0] * hidden if bias is None else numbers(bias)
    return norm


def conv_block(module, i, width):
    if module.in_channels != module.out_channels:
        raise Unsupported(f"layer {i}: Conv1d must keep its channels, as the state keeps its width")
    padding = module.padding
    same = padding == "same" or tuple(padding) == (module.kernel_size[0] // 2,)
    if (module.groups != 1 or tuple(module.stride) != (1,) or tuple(module.dilation) != (1,)
            or module.kernel_size[0] % 2 == 0 or not same or module.padding_mode != "zeros"):
        raise Unsupported(
            f"layer {i}: Conv1d must have an odd kernel, stride 1, no dilation or groups, "
            "and zero padding that keeps the length"
        )
    if width % module.in_channels != 0:
        raise Unsupported(f"layer {i}: {module.in_channels} channels do not divide the width {width}")
    return {
        "kind": "conv",
        "channels": module.in_channels,
        "kernel": module.kernel_size[0],
        "weight": numbers(module.weight.reshape(module.out_channels, -1)),
        "bias": bias_of(module),
    }


def residual_block(body, i, width):
    """The residual block of x + F(x) for the leaves of F, which must be one
    Linear, activation, Linear block that is a contraction."""
    fits = (len(body) == 3 and is_linear(body[0]) and is_linear(body[2])
            and body[0].in_features == width and body[2].out_features == width
            and body[0].out_features == body[2].in_features)
    if not fits:
        raise Unsupported(
            f"layer {i}: a residual connection must add a Linear({width}, M), activation, "
            f"Linear(M, {width}) to its input"
        )
    act = activation_of(body[1], i + 1)
    block = {
        "kind": "residual",
        "activation": act,
        "w1": numbers(body[0].weight),
        "b1": bias_of(body[0]),
        "w2": numbers(body[2].weight),
        "b2": bias_of(body[2]),
    }
    limit = contraction.bound(act, block["w1"], block["w2"])
    if limit >= contraction.LIMIT:
        raise Unsupported(
            f"layer {i}: the residual function is not a contraction: its Lipschitz bound is "
            f"{limit:.3f}, and weave inverts the block by iteration only below "
            f"{contraction.LIMIT}; shrink the weights (spectral norms) or the activation slope"
        )
    block["iters"] = contraction.iterations(limit)
    return block


def block_at(leaves, i, width):
    """The block that starts at leaves[i], and how many leaves it takes."""
    first = leaves[i]
    if kind(first) == "Residual":
        return residual_block(first.leaves, i, width), 1
    if kind(first) == "Conv1d":
        return conv_block(first, i, width), 1
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
    hidden = first.out_features
    normed = kind(leaves[i + 1]) in NORMS
    at = i + 2 if normed else i + 1
    if at >= len(leaves):
        raise Unsupported(f"layer {i}: a Linear and a norm with no activation is not reversible")
    act = activation_of(leaves[at], at)
    closes = at + 1 < len(leaves) and is_linear(leaves[at + 1])
    if normed and not (closes and leaves[at + 1].in_features == hidden):
        raise Unsupported(
            f"layer {i}: {kind(leaves[i + 1])} must be in a Linear, {kind(leaves[i + 1])}, "
            "activation, Linear block"
        )
    if closes and leaves[at + 1].in_features == hidden and leaves[at + 1].out_features == width:
        second = leaves[at + 1]
        block = {
            "kind": "mlp",
            "activation": act,
            "w1": numbers(first.weight),
            "b1": bias_of(first),
            "w2": numbers(second.weight),
            "b2": bias_of(second),
        }
        if normed:
            block["norm"] = norm_of(leaves[i + 1], i + 1, hidden)
        return block, at + 2 - i
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
    if block["kind"] in ("leapfrog", "conv"):
        return [first.weight, first.bias]
    if block["kind"] == "residual":
        one, two = first.leaves[0], first.leaves[2]
        return [one.weight, one.bias, two.weight, two.bias]
    normed = "norm" in block
    second = leaves[i + 3] if normed else leaves[i + 2]
    bound = [first.weight, first.bias, second.weight, second.bias]
    if normed:
        bound.append(leaves[i + 1].weight)
        if block["norm"].get("center"):
            bound.append(leaves[i + 1].bias)
    return bound


def export_bound(model, outputs=None, step=0.25, name="model", loss="mse", optimizer=None,
                 width=None, loss_scale=1):
    """The roop-weave model for a torch module as a dict, and the torch tensors
    that hold each of its weights, in the order roop-weave keeps them."""
    plain = kind(model) == "Sequential" and is_plain(model)
    leaves = flatten(model) if plain else traced_chain(model)
    opened = [m.leaves[0] if kind(m) == "Residual" and m.leaves else m for m in leaves]
    heads = [m for m in opened if is_linear(m) or kind(m) == "LinearAttention"]
    if kind(leaves[0]) == "Conv1d" and width is None:
        raise Unsupported("the state width is not known from a Conv1d first; pass width=")
    if not heads and width is None:
        raise Unsupported("the model has no Linear layer")
    if width is None:
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
    if loss_scale != 1:
        spec["loss_scale"] = loss_scale
    return spec, bound


def export(model, outputs=None, step=0.25, name="model", loss="mse", optimizer=None, width=None,
           loss_scale=1):
    """The roop-weave model for a torch module, as a dict. A `loss_scale` multiplies what the
    backward pass computes, and the optimizer divides it out, so that gradients below a
    unit of Q12 are not lost."""
    return export_bound(model, outputs, step, name, loss, optimizer, width, loss_scale)[0]


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
