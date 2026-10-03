"""The exporter on real torch modules. Set WEAVE_TORCH_OUT to a directory to
also write each exported model and the gradients torch finds for it, for the
Rust tests to compare."""

import json
import os
import unittest

import torch
from torch import nn

from torch_mirror import mirror
from torch_to_weave import Unsupported, export


def models():
    torch.manual_seed(0)
    return {
        "mlp_then_leapfrog": nn.Sequential(
            nn.Linear(4, 3), nn.ReLU(), nn.Linear(3, 4), nn.Linear(4, 4), nn.Tanh()
        ),
        "smooth_activations": nn.Sequential(
            nn.Linear(4, 3), nn.GELU(), nn.Linear(3, 4), nn.Linear(4, 5), nn.SiLU(),
            nn.Linear(4, 2), nn.Sigmoid(), nn.Linear(2, 4),
        ),
        "two_blocks_no_bias": nn.Sequential(
            nn.Linear(4, 5, bias=False), nn.Softsign(), nn.Linear(5, 4, bias=False),
            nn.Sequential(nn.Linear(4, 2), nn.Tanh(), nn.Linear(2, 4)),
        ),
    }


class RealTorch(unittest.TestCase):
    def test_the_weights_are_the_modules_weights(self):
        net = models()["mlp_then_leapfrog"]
        spec = export(net.eval())
        first = spec["layers"][0]
        self.assertEqual(first["kind"], "mlp")
        self.assertEqual(first["w1"], net[0].weight.tolist())
        self.assertEqual(first["b2"], net[2].bias.tolist())
        self.assertEqual(spec["layers"][1]["weight"], net[3].weight.tolist())

    def test_a_missing_bias_is_zeros(self):
        spec = export(models()["two_blocks_no_bias"])
        self.assertEqual(spec["layers"][0]["b1"], [0.0] * 5)

    def test_a_layer_weave_cannot_run_backward_is_refused(self):
        with self.assertRaisesRegex(Unsupported, "Softplus is not reversible"):
            export(nn.Sequential(nn.Linear(4, 4), nn.Softplus()))
        with self.assertRaisesRegex(Unsupported, "Conv1d must follow a Linear"):
            export(nn.Sequential(nn.Linear(4, 4), nn.ReLU(), nn.Conv1d(1, 1, 1)))

    def test_a_custom_module_is_traced_and_its_functions_are_activations(self):
        class Net(nn.Module):
            def __init__(self):
                super().__init__()
                self.up, self.down, self.tied = nn.Linear(4, 3), nn.Linear(3, 4), nn.Linear(4, 4)

            def forward(self, x):
                return torch.tanh(self.tied(self.down(torch.relu(self.up(x)))))

        spec = export(Net().eval())
        self.assertEqual([l["kind"] for l in spec["layers"]], ["mlp", "leapfrog"])
        self.assertEqual([l["activation"] for l in spec["layers"]], ["relu", "tanh"])

    def test_a_residual_connection_is_refused_because_the_graph_branches(self):
        class Residual(nn.Module):
            def __init__(self):
                super().__init__()
                self.f = nn.Linear(4, 4)

            def forward(self, x):
                return x + torch.relu(self.f(x))

        with self.assertRaisesRegex(Unsupported, "branches"):
            export(Residual())

    def test_export_for_each_model_and_its_autograd_gradients(self):
        out = os.environ.get("WEAVE_TORCH_OUT")
        for name, net in models().items():
            spec = export(net.eval(), outputs=2, name=name)
            loss, grads = mirror(spec)
            self.assertTrue(all(torch.isfinite(torch.tensor(g)).all() for g in grads))
            if out:
                with open(os.path.join(out, f"{name}.json"), "w") as f:
                    json.dump(spec, f)
                with open(os.path.join(out, f"{name}.grads.json"), "w") as f:
                    json.dump({"loss": loss, "grads": grads}, f)


if __name__ == "__main__":
    unittest.main()
