use crate::generate::{func_gen, vec_gen, Generatable, Generator, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LogitBias {
    values: BTreeMap<String, f32>,
}

impl ToJson for LogitBias {
    //TODO: I'm not sure this is the right format
    fn to_json(&self) -> serde_json::Value {
        json!(self.values)
    }
}

impl FromJson for LogitBias {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        let vs = v.as_object().unwrap();
        let values = vs
            .iter()
            .map(|(k, v)| (k.clone(), v.as_f64().unwrap() as f32))
            .collect();
        Ok(LogitBias { values })
    }
}

impl Generatable for LogitBias {
    fn gen(context: &mut GeneratorContext) -> LogitBias {
        let tuple_gen = func_gen(|c| (String::gen(c), c.rng.gen::<f32>()));
        let values = vec_gen(0, 10, tuple_gen).gen(context).into_iter().collect();
        LogitBias { values }
    }
}
