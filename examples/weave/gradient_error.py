"""How far weave's gradient is from the float gradient as training goes on.

Trains the digits network through weave, and after some epochs reads weave's gradient of
one batch (by an sgd step at a large rate on a copy, whose change in the weights over the
rate is the gradient) and compares it with the mirror's autograd gradient at the same weights.

    uv run python gradient_error.py --epochs 1,3,10,30
"""

import argparse
import copy

import torch
import torch.nn.functional as F

import data
import env
import models
from mirror_net import MirrorNet
from torch_to_weave import export
from weave_train import train

PROBE_RATE = 4.0


def gradients(net, xs, ys, classes, loss_scale):
    """Weave's gradient of the batch, and the mirror's, as flat vectors."""
    probe = copy.deepcopy(net)
    train(probe, xs, F.one_hot(ys, classes).double(), 1, PROBE_RATE, loss="softmax", roop=env.roop(),
          library=env.library(), loss_scale=loss_scale)
    ours = torch.cat([(a - b).detach().flatten() for a, b in zip(net.parameters(), probe.parameters())])
    mirror = MirrorNet(export(net, classes, loss="softmax")).double()
    loss = F.cross_entropy(mirror(xs), ys, reduction="sum")
    loss.backward()
    theirs = torch.cat([p.grad.flatten() for p in mirror.params])
    return ours / PROBE_RATE, theirs, float(loss.detach()) / len(ys)


def main():
    a = argparse.ArgumentParser()
    a.add_argument("--epochs", default="1,3,10,30")
    a.add_argument("--rate", type=float, default=0.005)
    a.add_argument("--act", default="tanh")
    a.add_argument("--seed", type=int, default=0)
    a.add_argument("--loss-scale", type=int, default=1)
    args = a.parse_args()
    torch.manual_seed(args.seed)
    xs, ys, _, _ = data.digits(args.seed)
    net = models.reversible(8, ("mlp",), 32, args.act).double()
    targets = F.one_hot(ys, 10).double()
    done = 0
    for epoch in [int(e) for e in args.epochs.split(",")]:
        train(net, xs, targets, epoch - done, args.rate, loss="softmax", roop=env.roop(),
              library=env.library(), loss_scale=args.loss_scale)
        done = epoch
        errors = []
        for start in range(0, 320, 32):
            ours, theirs, loss = gradients(net, xs[start : start + 32], ys[start : start + 32], 10,
                                            args.loss_scale)
            errors.append(((ours - theirs).norm() / theirs.norm(), ours.norm() / theirs.norm(), loss))
        e = torch.tensor(errors)
        print(f"epoch {epoch:3d}: batch loss {e[:, 2].mean():.4f}, relative error {e[:, 0].mean():.3f}, "
              f"norm ratio weave/float {e[:, 1].mean():.3f}")


if __name__ == "__main__":
    main()
