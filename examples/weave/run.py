"""Trains one configuration and prints one JSON line of results.

    uv run --extra torch python run.py digits --method weave --optimizer adam --rate 0.002

Methods: `plain` (an ordinary torch MLP), `mirror` (the reversible network in float,
torch's autograd), `weave` (the same network trained in Q12 by roop).
"""

import argparse
import json
import resource
import sys
import time

import torch
import torch.nn.functional as F

import data
import env
import models
from fixed_eval import fixed_accuracy
from mirror_net import MirrorNet
from torch_to_weave import export
from train_torch import fit, saved_bytes
from weave_train import train


def scores(net, x):
    with torch.no_grad():
        return net(x)


def metrics(logits, y):
    return {"acc": float((logits.argmax(1) == y).double().mean()),
            "ce": float(F.cross_entropy(logits, y))}


def main(argv=None):
    a = argparse.ArgumentParser()
    a.add_argument("task", choices=["digits", "chars"])
    a.add_argument("--method", default="weave", choices=["plain", "mirror", "weave"])
    a.add_argument("--optimizer", default="sgd", choices=["sgd", "momentum", "adam"])
    a.add_argument("--rate", type=float, default=0.01)
    a.add_argument("--epochs", type=int, default=10)
    a.add_argument("--seed", type=int, default=0)
    a.add_argument("--depth", type=int, default=8)
    a.add_argument("--hidden", type=int, default=32)
    a.add_argument("--act", default="tanh")
    a.add_argument("--kinds", default="mlp", help="comma list cycled through: mlp,attention,conv")
    a.add_argument("--cap", type=int, default=None, help="at most this many training samples")
    a.add_argument("--batch", type=int, default=data.BATCH)
    a.add_argument("--init-scale", type=float, default=1.0, help="scales the initial weights")
    a.add_argument("--out-scale", type=float, default=1.0,
                   help="scales the last weights of each block, to keep a deep state small")
    a.add_argument("--loss-scale", type=int, default=1,
                   help="weave multiplies the backward pass by this and the optimizer divides it out")
    a.add_argument("--norm", action="store_true", help="RMSNorm on the hidden layer of each perceptron")
    a.add_argument("--input-scale", type=float, default=1.0, help="scales the inputs")
    a.add_argument("--dtype", default="float64", choices=["float64", "float32"],
                   help="precision of the plain torch MLP (the mirror is always in doubles)")
    a.add_argument("--fixed-eval", action="store_true",
                   help="also run the test set through the compiled fixed point forward pass (weave only)")
    args = a.parse_args(argv)
    torch.manual_seed(args.seed)
    xs, ys, xt, yt = (data.digits(args.seed, cap=args.cap) if args.task == "digits"
                      else data.characters(args.seed, cap=args.cap or 2048))
    xs, xt = xs * args.input_scale, xt * args.input_scale
    classes = 10 if args.task == "digits" else 16
    if args.method == "plain":
        net = models.plain(classes)
    else:
        net = models.reversible(args.depth, tuple(args.kinds.split(",")), args.hidden, args.act,
                                 args.out_scale, args.norm)
        with torch.no_grad():
            for p in net.parameters():
                p.mul_(args.init_scale)
    dtype = getattr(torch, args.dtype) if args.method == "plain" else torch.float64
    net, xs, xt = net.to(dtype), xs.to(dtype), xt.to(dtype)
    out = {**vars(args), "train": len(ys), "test": len(yt),
           "params": sum(p.numel() for p in net.parameters())}
    start = time.perf_counter()
    if args.method == "weave":
        usage = {}
        targets = F.one_hot(ys, classes).double()
        curve = train(net, xs, targets, args.epochs, args.rate, loss="softmax",
                      optimizer={"kind": args.optimizer}, batch=args.batch, roop=env.roop(),
                      library=env.library(), usage=usage, loss_scale=args.loss_scale)
        out.update(usage)
        out["total_seconds"] = time.perf_counter() - start
        if args.fixed_eval:
            out["fixed_test_acc"] = fixed_accuracy(net, classes, xt, yt, env.library())
        net = MirrorNet(export(net, classes, loss="softmax"))
    else:
        if args.method == "mirror":
            net = MirrorNet(export(net, classes, loss="softmax")).double()
        used = resource.getrusage(resource.RUSAGE_SELF)
        before = used.ru_maxrss
        out["saved"] = saved_bytes(net, xs[: args.batch], ys[: args.batch])
        curve = fit(net, xs, ys, classes, args.epochs, args.rate, args.optimizer, args.batch)
        out["seconds"] = time.perf_counter() - start
        after = resource.getrusage(resource.RUSAGE_SELF)
        out["cpu"] = after.ru_utime + after.ru_stime - used.ru_utime - used.ru_stime
        out["rss"] = after.ru_maxrss
        out["rss_growth"] = out["rss"] - before
    out["curve"] = [round(c / len(ys), 5) for c in curve]
    out["train_metrics"] = metrics(scores(net, xs), ys)
    out["test_metrics"] = metrics(scores(net, xt), yt)
    json.dump(out, sys.stdout)
    print()


if __name__ == "__main__":
    main()
