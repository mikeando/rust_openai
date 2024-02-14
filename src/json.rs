use crate::types::error::Error;

pub trait ToJson {
    fn to_json(&self) -> serde_json::Value;
}

pub trait FromJson
where
    Self: Sized,
{
    fn from_json(v: &serde_json::Value) -> Result<Self, Error>;
}
