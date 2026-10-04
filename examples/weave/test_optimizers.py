"""weave's optimizers against torch's, on the same gradients: a few steps from the same
weights, with the model's own mirror finding the gradient in doubles."""

import copy
import unittest

import torch
from torch import nn

import env
from mirror_net import MirrorNet
from torch_to_weave import export
from train_torch import optimizer_for
from weave_train import train

BATCH = 4
STEPS = 3


def model():
    torch.manual_seed(3)
    return nn.Sequential(nn.Linear(4, 6), nn.Tanh(), nn.Linear(6, 4)).double()


def samples():
    torch.manual_seed(4)
    return torch.randn(BATCH * STEPS, 4, dtype=torch.float64), 0.5 * torch.randn(BATCH * STEPS, 1, dtype=torch.float64)


def torch_steps(net, kind, rate, xs, ts):
    mirror = MirrorNet(export(net, 1)).double()
    opt = optimizer_for(mirror.parameters(), kind, rate)
    for i in range(0, len(xs), BATCH):
        loss = 0.5 * ((mirror(xs[i : i + BATCH]) - ts[i : i + BATCH]) ** 2).sum()
        opt.zero_grad()
        loss.backward()
        opt.step()
    return [p.detach().flatten() for p in mirror.params]


def weave_steps(net, kind, rate, xs, ts):
    net = copy.deepcopy(net)
    train(net, xs, ts, 1, rate, optimizer={"kind": kind}, batch=BATCH, roop=env.roop(),
          library=env.library())
    return [p.detach().flatten() for p in net.parameters()]


class Optimizers(unittest.TestCase):
    def close(self, kind, rate, slack):
        net, (xs, ts) = model(), samples()
        start = [p.detach().flatten() for p in net.parameters()]
        ours, theirs = weave_steps(net, kind, rate, xs, ts), torch_steps(net, kind, rate, xs, ts)
        moved = sum(float((a - s).abs().sum()) for a, s in zip(theirs, start))
        gap = sum(float((a - b).abs().sum()) for a, b in zip(ours, theirs))
        self.assertGreater(moved, 0.1)
        self.assertLess(gap, slack * moved, f"{kind}: moved {moved:.4f}, differ by {gap:.4f}")

    def test_sgd_takes_torchs_steps(self):
        self.close("sgd", 0.0625, 0.05)

    def test_momentum_takes_torchs_steps(self):
        self.close("momentum", 0.0625, 0.05)

    def test_adam_takes_torchs_steps_with_the_bias_correction(self):
        self.close("adam", 0.0625, 0.05)


if __name__ == "__main__":
    unittest.main()
