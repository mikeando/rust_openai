use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use crate::types::{AssistantMessage, ToolMessage, UserMessage};
use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    UserMessage(UserMessage),
    AssistantMessage(AssistantMessage),
    ToolMessage(ToolMessage),
}

impl Message {
    pub fn role_as_string(&self) -> String {
        match self {
            Message::UserMessage(_) => "user".to_string(),
            Message::AssistantMessage(_) => "assistant".to_string(),
            Message::ToolMessage(_) => "tool".to_string(),
        }
    }

    pub fn user_message<T: Into<String>>(content: T) -> Message {
        Message::UserMessage(UserMessage {
            content: content.into(),
            name: None,
        })
    }

    pub fn as_assistant_message(&self) -> Option<&AssistantMessage> {
        if let Self::AssistantMessage(mesg) = self {
            Some(mesg)
        } else {
            None
        }
    }

    pub fn to_assistant_message(self) -> Option<AssistantMessage> {
        if let Self::AssistantMessage(mesg) = self {
            Some(mesg)
        } else {
            None
        }
    }
}

impl ToJson for Message {
    fn to_json(&self) -> serde_json::Value {
        match self {
            Message::UserMessage(m) => m.to_json(),
            Message::AssistantMessage(m) => m.to_json(),
            Message::ToolMessage(m) => m.to_json(),
        }
    }
}

impl FromJson for Message {
    fn from_json(v: &serde_json::Value) -> Result<Message, Error> {
        match v["role"].as_str() {
            Some("assistant") => Ok(Message::AssistantMessage(AssistantMessage::from_json(v)?)),
            Some("user") => Ok(Message::UserMessage(UserMessage::from_json(v)?)),
            Some("tool") => Ok(Message::ToolMessage(ToolMessage::from_json(v)?)),
            None => panic!("no role!"),
            Some(r) => panic!("Unknown role {}", r),
        }
    }
}

impl From<AssistantMessage> for Message {
    fn from(value: AssistantMessage) -> Self {
        Message::AssistantMessage(value)
    }
}

impl From<UserMessage> for Message {
    fn from(value: UserMessage) -> Self {
        Message::UserMessage(value)
    }
}

impl From<ToolMessage> for Message {
    fn from(value: ToolMessage) -> Self {
        Message::ToolMessage(value)
    }
}

impl Generatable for Message {
    fn gen(context: &mut GeneratorContext) -> Self {
        // Pick the enum type
        let enum_id = context.rng.gen_range(0..3);
        match enum_id {
            0 => Message::UserMessage(UserMessage::gen(context)),
            1 => Message::AssistantMessage(AssistantMessage::gen(context)),
            2 => Message::ToolMessage(ToolMessage::gen(context)),
            _ => unreachable!(),
        }
    }
}
