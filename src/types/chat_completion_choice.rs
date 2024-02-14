use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use crate::types::{FinishReason, Message};
use rand::Rng;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub logprobs: Option<Vec<f32>>,
    pub finish_reason: FinishReason,
    pub message: Message,
}

impl FromJson for ChatCompletionChoice {
    fn from_json(v: &serde_json::Value) -> Result<ChatCompletionChoice, Error> {
        let message = Message::from_json(&v["message"])?;
        let logprobs = v["logprobs"]
            .as_array()
            .map(|a| a.iter().map(|v| v.as_f64().unwrap() as f32).collect());

        Ok(ChatCompletionChoice {
            index: v["index"].as_i64().unwrap() as u32,
            logprobs,
            finish_reason: FinishReason::from_json(&v["finish_reason"])?,
            message,
        })
    }
}

impl ToJson for ChatCompletionChoice {
    fn to_json(&self) -> serde_json::Value {
        let mut v = json!({
            "index": self.index as i64,
            "finish_reason": self.finish_reason.to_json(),
            "message": self.message.to_json(),
        });
        if let Some(logprobs) = &self.logprobs {
            v["logprobs"] = json!(logprobs);
        }
        v
    }
}

impl Generatable for ChatCompletionChoice {
    fn gen(context: &mut GeneratorContext) -> Self {
        let logprobs = match context.rng.gen_bool(0.8) {
            true => None,
            false => Some(
                (0..context.rng.gen_range(0..10))
                    .map(|_| context.rng.gen::<f32>())
                    .collect(),
            ),
        };
        ChatCompletionChoice {
            index: context.rng.gen(),
            logprobs,
            finish_reason: context.gen(),
            message: context.gen(),
        }
    }
}
