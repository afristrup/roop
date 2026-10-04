use crate::{Tensor, quantize};

/// How a training step moves the weights.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Optimizer {
    Sgd,
    Momentum { beta: f64 },
    Adam { beta1: f64, beta2: f64 },
}

impl Optimizer {
    /// The state kept for each tensor: one tensor per name in the order of the
    /// calls, each named `m`, `v` or `t` and then the parameter's name.
    pub fn state(&self, tensor: &Tensor) -> Vec<Tensor> {
        let named = |prefix: &str| tensor.zeros_like(format!("{prefix}{}", tensor.name));
        match self {
            Self::Sgd => vec![],
            Self::Momentum { .. } => vec![named("m")],
            Self::Adam { .. } => vec![
                named("m"),
                named("v"),
                Tensor {
                    name: format!("t{}", tensor.name),
                    dims: vec![2],
                    data: vec![0.0; 2],
                },
            ],
        }
    }

    /// The call that updates a tensor from its gradient.
    pub fn update(&self, tensor: &Tensor) -> String {
        let (w, state) = (&tensor.name, self.state(tensor));
        let generics = match tensor.dims.as_slice() {
            [rows, cols] => format!("<{cols}, {rows}>"),
            [len] => format!("<{len}>"),
            _ => unreachable!("parameters are vectors or matrices"),
        };
        let shape = if tensor.dims.len() == 2 { "mat" } else { "vec" };
        // The vector versions are told the unit of the gradient; the matrix ones know it.
        let unit = if tensor.dims.len() == 1 { ", 1" } else { "" };
        let names: Vec<&str> = state.iter().map(|t| t.name.as_str()).collect();
        match self {
            Self::Sgd => format!("    call sgd_{shape}{generics}({w}, g{w}, lr);\n"),
            Self::Momentum { beta } => format!(
                "    call momentum_{shape}{generics}({w}, g{w}, {}, lr, {}{unit});\n",
                names[0],
                quantize(*beta)
            ),
            Self::Adam { beta1, beta2 } => format!(
                "    call adam_{shape}{generics}({w}, g{w}, {}, {}, {}, lr, {}, {}{unit});\n",
                names[0],
                names[1],
                names[2],
                quantize(*beta1),
                quantize(*beta2)
            ),
        }
    }
}
