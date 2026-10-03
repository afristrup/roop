use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum WeaveError {
    Json(String),
    Field {
        path: String,
        expected: &'static str,
    },
    Shape {
        name: String,
        expected: String,
        found: String,
    },
    Activation {
        path: String,
        name: String,
    },
    Layer {
        index: usize,
        kind: String,
    },
    NoLayers,
    Outputs {
        outputs: usize,
        width: usize,
    },
    Name(String),
    Loss(String),
    Optimizer(String),
}

impl fmt::Display for WeaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Json(message) => write!(f, "the model is not JSON: {message}"),
            Self::Field { path, expected } => write!(f, "`{path}` must be {expected}"),
            Self::Shape {
                name,
                expected,
                found,
            } => {
                write!(f, "`{name}` must be {expected}, and it is {found}")
            }
            Self::Activation { path, name } => write!(
                f,
                "`{path}`: no activation `{name}`; weave has identity, cauchy, softsign, relu, tanh, sigmoid, silu and gelu"
            ),
            Self::Layer { index, kind } => write!(
                f,
                "layer {index}: no layer `{kind}`; weave layers are leapfrog and mlp, and a layer that is not reversible cannot be run backward"
            ),
            Self::NoLayers => write!(f, "the model has no layers"),
            Self::Outputs { outputs, width } => {
                write!(f, "{outputs} outputs from a state of width {width}")
            }
            Self::Loss(name) => write!(f, "no loss `{name}`; weave has mse, sigmoid and softmax"),
            Self::Optimizer(name) => {
                write!(f, "no optimizer `{name}`; weave has sgd, momentum and adam")
            }
            Self::Name(name) => write!(f, "`{name}` is not a name for roop functions"),
        }
    }
}

impl std::error::Error for WeaveError {}
