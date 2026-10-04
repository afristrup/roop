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


def model(blocks=1, shrink=1.0):
    torch.manual_seed(3)
    net = nn.Sequential(*[m for _ in range(blocks) for m in (nn.Linear(4, 6), nn.Tanh(), nn.Linear(6, 4))])
    with torch.no_grad():
        for p in net.parameters():
            p.mul_(shrink)
    return net.double()


def samples(steps):
    torch.manual_seed(4)
    return torch.randn(BATCH * steps, 4, dtype=torch.float64), 0.5 * torch.randn(BATCH * steps, 1, dtype=torch.float64)


def torch_steps(net, kind, rate, xs, ts):
    mirror = MirrorNet(export(net, 1)).double()
    opt = optimizer_for(mirror.parameters(), kind, rate)
    for i in range(0, len(xs), BATCH):
        loss = 0.5 * ((mirror(xs[i : i + BATCH]) - ts[i : i + BATCH]) ** 2).sum()
        opt.zero_grad()
        loss.backward()
        opt.step()
    return [p.detach().flatten() for p in mirror.params]


def weave_steps(net, kind, rate, xs, ts, loss_scale=1):
    net = copy.deepcopy(net)
    train(net, xs, ts, 1, rate, optimizer={"kind": kind}, batch=BATCH, roop=env.roop(),
          library=env.library(), loss_scale=loss_scale)
    return [p.detach().flatten() for p in net.parameters()]


class Optimizers(unittest.TestCase):
    def distance(self, kind, rate, steps=3, scale=1, blocks=1, shrink=1.0):
        """How far weave's weights end from torch's, and how far torch's moved."""
        net, (xs, ts) = model(blocks, shrink), samples(steps)
        start = [p.detach().flatten() for p in net.parameters()]
        ours = weave_steps(net, kind, rate, xs, ts, scale)
        theirs = torch_steps(net, kind, rate, xs, ts)
        moved = sum(float((a - s).abs().sum()) for a, s in zip(theirs, start))
        gap = sum(float((a - b).abs().sum()) for a, b in zip(ours, theirs))
        return gap, moved

    def close(self, kind, rate, slack, steps=3):
        gap, moved = self.distance(kind, rate, steps)
        self.assertGreater(moved, 0.05)
        self.assertLess(gap, slack * moved, f"{kind}: moved {moved:.4f}, differ by {gap:.4f}")

    def test_sgd_takes_torchs_steps(self):
        self.close("sgd", 0.0625, 0.05)

    def test_a_step_below_a_unit_of_the_weight_is_not_lost(self):
        self.close("sgd", 1 / 4096, 0.3, steps=24)
        self.close("momentum", 1 / 4096, 0.3, steps=24)

    def test_a_loss_scale_keeps_the_gradients_that_q12_would_lose(self):
        # Two blocks of weights near 0.05: the gradient reaching the first is below a few
        # units of Q12, and Adam turns the noise that is left into full steps.
        gap, moved = self.distance("adam", 0.0625, steps=4, blocks=2, shrink=0.05)
        self.assertGreater(gap, 0.3 * moved)
        gap, moved = self.distance("adam", 0.0625, steps=4, blocks=2, shrink=0.05, scale=256)
        self.assertLess(gap, 0.05 * moved)

    def test_momentum_takes_torchs_steps(self):
        self.close("momentum", 0.0625, 0.05)

    def test_adam_takes_torchs_steps_with_the_bias_correction(self):
        self.close("adam", 0.0625, 0.05)


if __name__ == "__main__":
    unittest.main()
