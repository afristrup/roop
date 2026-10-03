/// The pointwise function of a layer, with the same formulas as
/// `roop/weave/act.roop` but in doubles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activation {
    Identity,
    Cauchy,
    Softsign,
    Relu,
    Tanh,
}

impl Activation {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "identity" => Some(Self::Identity),
            "cauchy" => Some(Self::Cauchy),
            "softsign" => Some(Self::Softsign),
            "relu" => Some(Self::Relu),
            "tanh" => Some(Self::Tanh),
            _ => None,
        }
    }

    /// The kind number the weave library switches on.
    pub fn kind(self) -> i64 {
        self as i64
    }

    pub fn eval(self, z: f64) -> f64 {
        match self {
            Self::Identity => z,
            Self::Cauchy => z / (1.0 + z * z),
            Self::Softsign => z / (1.0 + z.abs()),
            Self::Relu => z.max(0.0),
            Self::Tanh if z.abs() > 3.0 => z.signum(),
            Self::Tanh => z * (27.0 + z * z) / (27.0 + 9.0 * z * z),
        }
    }
}
