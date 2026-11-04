#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Invalid { file: String, message: String },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
