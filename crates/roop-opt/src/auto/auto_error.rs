use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum AutoError {
    StartsNonZero { name: String },
    UnknownRegion { name: String, region: String },
    NotARegion { region: String },
}

impl fmt::Display for AutoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StartsNonZero { name } => write!(
                f,
                "`auto ancilla {name}` must start at zero, since what is kept comes back as zero"
            ),
            Self::UnknownRegion { name, region } => write!(
                f,
                "`auto<'{region}> ancilla {name}`: no loop or block labeled '{region} after it"
            ),
            Self::NotARegion { region } => {
                write!(
                    f,
                    "'{region} labels something that is not a loop or a block"
                )
            }
        }
    }
}

impl std::error::Error for AutoError {}
