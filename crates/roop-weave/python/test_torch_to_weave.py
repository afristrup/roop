"""Tests of the exporter against stand-ins for torch.nn, so that they run
without torch. Set WEAVE_EXPORT_OUT to also write the exported model as JSON."""

import json
import os
import unittest

from torch_to_weave import Unsupported, export


class Tensor:
    def __init__(self, rows):
        self.rows = rows

    def detach(self):
        return self

    def cpu(self):
        return self

    def tolist(self):
        return self.rows


class Module:
    def children(self):
        return iter(())


class Sequential(Module):
    def __init__(self, *modules):
        self.modules = modules

    def children(self):
        return iter(self.modules)


class Linear(Module):
    def __init__(self, n_in, n_out, bias=True):
        self.in_features, self.out_features = n_in, n_out
        self.weight = Tensor([[((i * 7 + j * 3) % 9 - 4) / 10 for j in range(n_in)] for i in range(n_out)])
        self.bias = Tensor([((i * 2) % 5 - 2) / 10 + 0.05 for i in range(n_out)]) if bias else None


class ReLU(Module):
    pass


class Tanh(Module):
    pass


class Softsign(Module):
    pass


class Identity(Module):
    pass


class GELU(Module):
    pass


class Dropout(Module):
    pass


class Export(unittest.TestCase):
    def test_linear_act_linear_is_an_mlp_block(self):
        spec = export(Sequential(Linear(4, 3), ReLU(), Linear(3, 4)))
        self.assertEqual(spec["width"], 4)
        self.assertEqual([layer["kind"] for layer in spec["layers"]], ["mlp"])
        layer = spec["layers"][0]
        self.assertEqual(layer["activation"], "relu")
        self.assertEqual((len(layer["w1"]), len(layer["w1"][0])), (3, 4))
        self.assertEqual((len(layer["w2"]), len(layer["w2"][0])), (4, 3))

    def test_linear_act_alone_is_a_leapfrog_layer(self):
        spec = export(Sequential(Linear(4, 5), Tanh()))
        layer = spec["layers"][0]
        self.assertEqual(layer["kind"], "leapfrog")
        self.assertEqual((len(layer["weight"]), len(layer["weight"][0])), (5, 4))
        self.assertEqual(len(layer["bias"]), 5)

    def test_blocks_follow_each_other_and_nested_sequentials_are_flattened(self):
        model = Sequential(
            Sequential(Linear(4, 3), Softsign(), Linear(3, 4)),
            Linear(4, 4),
            Identity(),
            Sequential(Linear(4, 2), Tanh(), Linear(2, 4)),
        )
        spec = export(model, outputs=2, step=0.5, name="net")
        self.assertEqual([l["kind"] for l in spec["layers"]], ["mlp", "leapfrog", "mlp"])
        self.assertEqual((spec["outputs"], spec["step"], spec["name"]), (2, 0.5, "net"))

    def test_a_missing_bias_is_zero(self):
        spec = export(Sequential(Linear(4, 3, bias=False), ReLU()))
        self.assertEqual(spec["layers"][0]["bias"], [0.0, 0.0, 0.0])

    def test_an_activation_weave_lacks_is_refused_by_name(self):
        with self.assertRaisesRegex(Unsupported, "GELU is not reversible"):
            export(Sequential(Linear(4, 4), GELU()))

    def test_a_linear_with_no_activation_is_refused(self):
        with self.assertRaisesRegex(Unsupported, "no activation"):
            export(Sequential(Linear(4, 4)))

    def test_a_layer_that_changes_the_width_is_refused(self):
        with self.assertRaisesRegex(Unsupported, "state is 4 wide"):
            export(Sequential(Linear(4, 3), ReLU(), Linear(3, 4), Linear(5, 4), ReLU()))

    def test_a_layer_that_is_not_linear_is_refused(self):
        with self.assertRaisesRegex(Unsupported, "Dropout must follow a Linear"):
            export(Sequential(Linear(4, 4), ReLU(), Dropout()))

    def test_a_model_without_linear_layers_is_refused(self):
        with self.assertRaisesRegex(Unsupported, "no Linear"):
            export(Sequential(ReLU()))

    def test_the_result_is_json(self):
        spec = export(Sequential(Linear(4, 3), Tanh(), Linear(3, 4), Linear(4, 4), Tanh()), outputs=1)
        text = json.dumps(spec)
        self.assertEqual(json.loads(text), spec)
        path = os.environ.get("WEAVE_EXPORT_OUT")
        if path:
            with open(path, "w") as out:
                out.write(text)


if __name__ == "__main__":
    unittest.main()
