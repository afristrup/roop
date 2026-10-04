"""The exporter on real torch modules. Set WEAVE_TORCH_OUT to a directory to
also write each exported model and the gradients torch finds for it, for the
Rust tests to compare."""

import json
import os
import unittest

import torch
from torch import nn

from torch_mirror import evaluate, mirror, sample
import contraction
from weave_train import train
from torch_to_weave import Unsupported, export
from weave_modules import LinearAttention


LOSSES = {"softmax_head": "softmax", "sigmoid_head": "sigmoid"}


class ResidualBlock(nn.Module):
    """x + F(x) with F a Linear, activation, Linear, its weights shrunk to a contraction."""

    def __init__(self, width=4, hidden=3, act=nn.Tanh, scale=0.8):
        super().__init__()
        self.f = nn.Sequential(nn.Linear(width, hidden), act(), nn.Linear(hidden, width))
        with torch.no_grad():
            for layer in (self.f[0], self.f[2]):
                layer.weight.mul_(scale)

    def forward(self, x):
        return x + self.f(x)


def models():
    torch.manual_seed(0)
    return {
        "layernorm_blocks": nn.Sequential(
            nn.Linear(4, 5), nn.LayerNorm(5, eps=1e-2), nn.GELU(), nn.Linear(5, 4),
            nn.Linear(4, 3), nn.LayerNorm(3, eps=1e-2, elementwise_affine=False), nn.Tanh(), nn.Linear(3, 4),
            nn.Linear(4, 3), nn.LayerNorm(3, eps=1e-2, bias=False), nn.ReLU(), nn.Linear(3, 4),
        ),
        "residual_blocks": nn.Sequential(ResidualBlock(4, 3), ResidualBlock(4, 5, nn.GELU)),
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

    def test_training_a_residual_net_and_a_layernorm_net_through_weave(self):
        xs = [[1, 1, 1, 0], [1, -1, 1, 0], [-1, 1, 1, 0], [-1, -1, 1, 0]]
        ts = [[-0.5], [0.5], [0.5], [-0.5]]
        roop = os.environ.get("WEAVE_ROOP", "roop")
        torch.manual_seed(2)
        nets = [
            nn.Sequential(*[ResidualBlock(4, 6) for _ in range(3)]),
            nn.Sequential(*[m for _ in range(3) for m in (
                nn.Linear(4, 6), nn.LayerNorm(6, eps=1e-2), nn.Tanh(), nn.Linear(6, 4))]),
        ]
        for net in nets:
            before = [p.detach().clone() for p in net.parameters()]
            kept = 0.8 if isinstance(net[0], ResidualBlock) else None
            losses = train(net, xs, ts, epochs=1500, rate=0.05, roop=roop, keep_contraction=kept)
            self.assertLess(losses[-1], losses[0] / 5)
            self.assertEqual(len(losses), 1500)
            self.assertTrue(all(not torch.equal(a, b) for a, b in zip(before, net.parameters())))
            again = evaluate(export(net.eval(), outputs=1), xs, ts)
            if kept is None:
                self.assertAlmostEqual(again, losses[-1], delta=0.02 + 0.1 * losses[-1])
            else:
                # The last stretch's weights were scaled back to the bound after it.
                self.assertLess(again, losses[0] / 2)

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

    def test_layernorm_is_a_centered_norm_with_a_bias(self):
        net = nn.Sequential(nn.Linear(4, 5), nn.LayerNorm(5, eps=1e-2), nn.ReLU(), nn.Linear(5, 4))
        norm = export(net)["layers"][0]["norm"]
        self.assertEqual((norm["eps"], norm["center"]), (1e-2, True))
        self.assertEqual(norm["bias"], net[1].bias.tolist())
        self.assertEqual(norm["gain"], net[1].weight.tolist())
        plain = nn.Sequential(nn.Linear(4, 5), nn.LayerNorm(5, eps=1e-2, elementwise_affine=False),
                              nn.ReLU(), nn.Linear(5, 4))
        norm = export(plain)["layers"][0]["norm"]
        self.assertEqual((norm["gain"], norm["bias"]), ([1.0] * 5, [0.0] * 5))

    def test_layernorm_with_an_epsilon_below_the_grid_or_out_of_place_is_refused(self):
        with self.assertRaisesRegex(Unsupported, "LayerNorm eps .* below 1/4096"):
            export(nn.Sequential(nn.Linear(4, 5), nn.LayerNorm(5, eps=1e-5), nn.ReLU(), nn.Linear(5, 4)))
        with self.assertRaisesRegex(Unsupported, "LayerNorm must be in a Linear"):
            export(nn.Sequential(nn.Linear(4, 5), nn.LayerNorm(5, eps=1e-2), nn.ReLU()))

    def test_x_plus_f_of_x_is_a_residual_block(self):
        net = nn.Sequential(ResidualBlock(4, 3), nn.Linear(4, 3), nn.Tanh(), nn.Linear(3, 4), ResidualBlock(4, 5))
        spec = export(net.eval())
        self.assertEqual([l["kind"] for l in spec["layers"]], ["residual", "mlp", "residual"])
        block = spec["layers"][0]
        self.assertEqual(block["w1"], net[0].f[0].weight.tolist())
        self.assertEqual(block["b2"], net[0].f[2].bias.tolist())
        bound = contraction.bound("tanh", block["w1"], block["w2"])
        self.assertLess(bound, 0.9)
        self.assertEqual(block["iters"], contraction.iterations(bound))

    def test_a_traced_residual_module_and_a_residual_in_the_middle_of_a_chain(self):
        class Net(nn.Module):
            def __init__(self):
                super().__init__()
                self.up, self.down = nn.Linear(4, 3), nn.Linear(3, 4)
                self.mix = nn.Linear(4, 4)
                self.res = ResidualBlock()

            def forward(self, x):
                y = self.mix(x)
                y = torch.tanh(y)
                return self.res(y)

        spec = export(Net().eval())
        self.assertEqual([l["kind"] for l in spec["layers"]], ["leapfrog", "residual"])

    def test_the_weights_of_a_residual_block_are_the_modules_weights_in_the_mirrors_order(self):
        spec = export(models()["residual_blocks"].eval())
        loss, grads = mirror(spec)
        self.assertEqual([len(g) for g in grads], [12, 3, 12, 4, 20, 5, 20, 4])

    def test_a_residual_function_that_is_not_a_contraction_is_refused(self):
        with self.assertRaisesRegex(Unsupported, "not a contraction"):
            export(nn.Sequential(ResidualBlock(4, 3, scale=8.0)))

    def test_a_residual_connection_that_is_not_x_plus_a_perceptron_is_refused(self):
        class OneLinear(nn.Module):
            def __init__(self):
                super().__init__()
                self.f = nn.Linear(4, 4)

            def forward(self, x):
                return x + torch.relu(self.f(x))

        class TwoPaths(nn.Module):
            def __init__(self):
                super().__init__()
                self.a, self.b = nn.Linear(4, 4), nn.Linear(4, 4)

            def forward(self, x):
                return torch.relu(self.a(x)) + torch.tanh(self.b(x))

        class Doubled(nn.Module):
            def forward(self, x):
                return x + x

        with self.assertRaisesRegex(Unsupported, "must add a Linear"):
            export(OneLinear())
        with self.assertRaisesRegex(Unsupported, "branches"):
            export(TwoPaths())
        with self.assertRaises(Unsupported):
            export(Doubled(), width=4)

    def test_the_mirror_gradients_of_a_residual_net_are_those_of_the_module_itself(self):
        torch.manual_seed(3)
        net = nn.Sequential(ResidualBlock(4, 3, nn.ReLU, 0.8), ResidualBlock(4, 5, nn.ReLU, 0.8))
        with torch.no_grad():
            for tensor in net.parameters():
                tensor.copy_(torch.round(tensor * 4096) / 4096)
        spec = export(net.eval(), outputs=2)
        loss, grads = mirror(spec)
        x, t = sample(4, 2)
        net = net.double()
        value = 0.5 * ((net(x)[:2] - t) ** 2).sum()
        value.backward()
        ours = [g for layer in grads for g in layer]
        theirs = []
        for block in net:
            for layer in (block.f[0], block.f[2]):
                theirs += layer.weight.grad.flatten().tolist() + layer.bias.grad.tolist()
        # The activation is exact, so the two differ only by the remnant of the
        # fixed-point iteration, which is far below 2^-12.
        self.assertAlmostEqual(loss, value.item(), delta=1e-6)
        self.assertEqual(len(ours), len(theirs))
        for a, b in zip(ours, theirs):
            self.assertAlmostEqual(a, b, delta=1e-5)

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
