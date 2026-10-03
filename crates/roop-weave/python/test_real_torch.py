"""The exporter on real torch modules. Set WEAVE_TORCH_OUT to a directory to
also write each exported model and the gradients torch finds for it, for the
Rust tests to compare."""

import json
import os
import unittest

import torch
from torch import nn

from torch_mirror import evaluate, mirror
from weave_train import train
from torch_to_weave import Unsupported, export
from weave_modules import LinearAttention


LOSSES = {"softmax_head": "softmax", "sigmoid_head": "sigmoid"}


def models():
    torch.manual_seed(0)
    return {
        "softmax_head": nn.Sequential(
            nn.Linear(4, 3), nn.ReLU(), nn.Linear(3, 4), nn.Linear(4, 3), nn.Tanh(), nn.Linear(3, 4),
        ),
        "sigmoid_head": nn.Sequential(nn.Linear(4, 3), nn.Tanh(), nn.Linear(3, 4)),
        "mlp_then_leapfrog": nn.Sequential(
            nn.Linear(4, 3), nn.ReLU(), nn.Linear(3, 4), nn.Linear(4, 4), nn.Tanh()
        ),
        "smooth_activations": nn.Sequential(
            nn.Linear(4, 3), nn.GELU(), nn.Linear(3, 4), nn.Linear(4, 5), nn.SiLU(),
            nn.Linear(4, 2), nn.Sigmoid(), nn.Linear(2, 4),
        ),
        "conv_blocks": nn.Sequential(
            nn.Linear(6, 4), nn.Tanh(), nn.Linear(4, 6),
            nn.Conv1d(2, 2, 3, padding=1), nn.Conv1d(3, 3, 3, padding="same", bias=False),
        ),
        "rmsnorm_blocks": nn.Sequential(
            nn.Linear(4, 5), nn.RMSNorm(5, eps=1e-2), nn.GELU(), nn.Linear(5, 4),
            nn.Linear(4, 3), nn.RMSNorm(3, eps=1e-2, elementwise_affine=False), nn.Tanh(), nn.Linear(3, 4),
        ),
        "attention_and_mlp": nn.Sequential(
            LinearAttention(2, 2), nn.Linear(4, 3), nn.Tanh(), nn.Linear(3, 4), LinearAttention(2, 2),
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
        with self.assertRaisesRegex(Unsupported, "BatchNorm1d must follow a Linear"):
            export(nn.Sequential(nn.Linear(4, 4), nn.ReLU(), nn.BatchNorm1d(4)))

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

    def test_attention_is_a_block_and_a_traced_module_may_hold_it(self):
        class Net(nn.Module):
            def __init__(self):
                super().__init__()
                self.attn, self.up, self.down = LinearAttention(2, 2), nn.Linear(4, 3), nn.Linear(3, 4)

            def forward(self, x):
                return self.down(torch.relu(self.up(self.attn(x))))

        spec = export(Net())
        self.assertEqual([l["kind"] for l in spec["layers"]], ["attention", "mlp"])
        self.assertEqual(spec["width"], 4)

    def test_attention_must_fill_the_state(self):
        with self.assertRaisesRegex(Unsupported, "reads 6 numbers"):
            export(nn.Sequential(nn.Linear(4, 3), nn.ReLU(), nn.Linear(3, 4), LinearAttention(3, 2)))

    def test_training_through_weave_puts_the_trained_weights_back(self):
        torch.manual_seed(1)
        net = nn.Sequential(
            nn.Linear(4, 6), nn.Tanh(), nn.Linear(6, 4),
            nn.Linear(4, 6), nn.Tanh(), nn.Linear(6, 4),
            nn.Linear(4, 6), nn.Tanh(), nn.Linear(6, 4),
        )
        xs = [[1, 1, 1, 0], [1, -1, 1, 0], [-1, 1, 1, 0], [-1, -1, 1, 0]]
        ts = [[-0.5], [0.5], [0.5], [-0.5]]
        before = net[0].weight.detach().clone()
        roop = os.environ.get("WEAVE_ROOP", "roop")
        losses = train(net, xs, ts, epochs=1500, rate=0.05, roop=roop)
        self.assertLess(losses[-1], losses[0] / 5)
        self.assertFalse(torch.equal(before, net[0].weight))
        # The weights that came back reproduce the loss weave reported.
        again = evaluate(export(net.eval(), outputs=1), xs, ts)
        self.assertAlmostEqual(again, losses[-1], delta=0.02 + 0.1 * losses[-1])

    def test_conv1d_is_a_block_when_it_keeps_the_length_and_the_channels(self):
        spec = export(nn.Sequential(nn.Linear(6, 4), nn.ReLU(), nn.Linear(4, 6), nn.Conv1d(2, 2, 3, padding=1)))
        conv = spec["layers"][-1]
        self.assertEqual((conv["kind"], conv["channels"], conv["kernel"]), ("conv", 2, 3))
        self.assertEqual((len(conv["weight"]), len(conv["weight"][0])), (2, 6))
        for bad in [nn.Conv1d(2, 3, 3, padding=1), nn.Conv1d(2, 2, 3), nn.Conv1d(2, 2, 3, stride=2, padding=1),
                    nn.Conv1d(2, 2, 2, padding=1), nn.Conv1d(2, 2, 3, padding=1, groups=2)]:
            with self.assertRaises(Unsupported):
                export(nn.Sequential(nn.Linear(6, 4), nn.ReLU(), nn.Linear(4, 6), bad))

    def test_a_conv_first_needs_the_width(self):
        net = nn.Sequential(nn.Conv1d(2, 2, 3, padding=1))
        with self.assertRaisesRegex(Unsupported, "pass width"):
            export(net)
        self.assertEqual(export(net, width=6)["width"], 6)

    def test_rmsnorm_inside_a_perceptron_is_a_norm_of_the_block(self):
        net = nn.Sequential(nn.Linear(4, 5), nn.RMSNorm(5, eps=1e-2), nn.ReLU(), nn.Linear(5, 4))
        spec = export(net)
        self.assertEqual(spec["layers"][0]["norm"]["eps"], 1e-2)
        self.assertEqual(len(spec["layers"][0]["norm"]["gain"]), 5)

    def test_rmsnorm_with_an_epsilon_below_the_grid_or_out_of_place_is_refused(self):
        with self.assertRaisesRegex(Unsupported, "below 1/4096"):
            export(nn.Sequential(nn.Linear(4, 5), nn.RMSNorm(5), nn.ReLU(), nn.Linear(5, 4)))
        with self.assertRaisesRegex(Unsupported, "RMSNorm must be in a Linear"):
            export(nn.Sequential(nn.Linear(4, 5), nn.RMSNorm(5, eps=1e-2), nn.ReLU()))

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
            spec = export(net.eval(), outputs=3 if name in LOSSES else 2, name=name,
                          loss=LOSSES.get(name, "mse"))
            loss, grads = mirror(spec)
            self.assertTrue(all(torch.isfinite(torch.tensor(g)).all() for g in grads))
            if out:
                with open(os.path.join(out, f"{name}.json"), "w") as f:
                    json.dump(spec, f)
                with open(os.path.join(out, f"{name}.grads.json"), "w") as f:
                    json.dump({"loss": loss, "grads": grads}, f)


if __name__ == "__main__":
    unittest.main()
