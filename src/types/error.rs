#[derive(Debug)]
pub enum Error {
    InvalidJsonStructure,
    InvalidModelName,
    InvalidFinishReason,
    InvalidResponseFormat,
    InvalidToolChoice,
    JsonExpectedArray,
    JsonExpectedObject,
    JsonExpectedI64,
    JsonExpectedF64,
    JsonExpectedString,
    Wrapped(anyhow::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Error::Wrapped(value)
    }
}
