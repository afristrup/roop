/// The pointwise function of a layer, with the same formulas as
/// `roop/weave/act.roop` but in doubles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activation {
    Identity,
    Cauchy,
    Softsign,
    Relu,
    Tanh,
    Sigmoid,
    Silu,
    Gelu,
}

/// The 1.702 of the usual approximation of GELU, as weave holds it.
const GELU_SCALE: f64 = 6971.0 / 4096.0;

/// tanh as a Pade approximation that is 1 at 3 and stays there.
fn pade(z: f64) -> f64 {
    match z.abs() > 3.0 {
        true => z.signum(),
        false => z * (27.0 + z * z) / (27.0 + 9.0 * z * z),
    }
}

fn sigmoid(z: f64) -> f64 {
    0.5 + 0.5 * pade(z / 2.0)
}

impl Activation {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "identity" => Some(Self::Identity),
            "cauchy" => Some(Self::Cauchy),
            "softsign" => Some(Self::Softsign),
            "relu" => Some(Self::Relu),
            "tanh" => Some(Self::Tanh),
            "sigmoid" => Some(Self::Sigmoid),
            "silu" => Some(Self::Silu),
            "gelu" => Some(Self::Gelu),
            _ => None,
        }
    }

    /// The kind number the weave library switches on.
    pub fn kind(self) -> i64 {
        self as i64
    }

    /// A bound on the slope of the function, found by scanning it.
    pub fn lipschitz(self) -> f64 {
        match self {
            Self::Sigmoid => 0.2501,
            Self::Silu | Self::Gelu => 1.1205,
            _ => 1.0,
        }
    }

    pub fn eval(self, z: f64) -> f64 {
        match self {
            Self::Identity => z,
            Self::Cauchy => z / (1.0 + z * z),
            Self::Softsign => z / (1.0 + z.abs()),
            Self::Relu => z.max(0.0),
            Self::Tanh => pade(z),
            Self::Sigmoid => sigmoid(z),
            Self::Silu => z * sigmoid(z),
            Self::Gelu => z * sigmoid(GELU_SCALE * z),
        }
    }
}
