"""The digits experiment, for a few epochs: the loss falls through weave, and weave and
the float mirror of the same network fall alike."""

import contextlib
import io
import json
import unittest

import run


def experiment(*args):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        run.main(["digits", "--epochs", "6", "--depth", "4", "--cap", "480", *args])
    return json.loads(out.getvalue())


class Digits(unittest.TestCase):
    def test_the_loss_falls_through_weave(self):
        for optimizer, rate in [("sgd", "0.005"), ("adam", "0.001")]:
            curve = experiment("--method", "weave", "--optimizer", optimizer, "--rate", rate)["curve"]
            self.assertLess(curve[-1], curve[0] / 1.5, f"{optimizer}: {curve}")

    def test_weave_and_the_mirror_reach_alike_losses(self):
        ours = experiment("--method", "weave", "--optimizer", "sgd", "--rate", "0.005")
        theirs = experiment("--method", "mirror", "--optimizer", "sgd", "--rate", "0.005")
        self.assertAlmostEqual(ours["curve"][-1], theirs["curve"][-1], delta=0.1)
        self.assertGreater(ours["test_metrics"]["acc"], 0.5)


if __name__ == "__main__":
    unittest.main()
