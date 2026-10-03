use crate::{Layer, Tensor, quantize};

/// A network of reversible layers on a state (q, p) of `width` numbers each.
/// It reads its input into q, with p zero, and its output is the first
/// `outputs` numbers of q.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub name: String,
    pub width: usize,
    pub outputs: usize,
    pub step: f64,
    pub layers: Vec<Layer>,
}

impl Model {
    pub fn tensors(&self) -> Vec<&Tensor> {
        self.layers.iter().flat_map(Layer::tensors).collect()
    }

    pub fn tensors_mut(&mut self) -> Vec<&mut Tensor> {
        self.layers
            .iter_mut()
            .flat_map(Layer::tensors_mut)
            .collect()
    }

    /// The tensors the gradients go into, in the same order, each named `g` and
    /// the parameter's name.
    pub fn gradients(&self) -> Vec<Tensor> {
        let named = |t: &&Tensor| t.zeros_like(format!("g{}", t.name));
        self.tensors().iter().map(named).collect()
    }

    /// For each layer, whether it is a perceptron that adds into q. Perceptrons
    /// alternate, starting with q; the entries of other layers are false.
    pub fn adds_into_q(&self) -> Vec<bool> {
        let mut seen = 0;
        let mut flags = Vec::new();
        for layer in &self.layers {
            let is_mlp = matches!(layer, Layer::Mlp { .. });
            flags.push(is_mlp && seen % 2 == 0);
            seen += is_mlp as usize;
        }
        flags
    }

    /// The step size and every number as weave holds them, on the 1/4096 grid.
    pub fn snapped(&self) -> Self {
        let on_grid = |x: f64| quantize(x) as f64 / 4096.0;
        let mut model = self.clone();
        model.step = on_grid(model.step);
        for tensor in model.tensors_mut() {
            tensor.data.iter_mut().for_each(|x| *x = on_grid(*x));
        }
        model
    }
}
