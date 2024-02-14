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
}
