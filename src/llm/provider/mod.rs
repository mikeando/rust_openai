use crate::llm::client::RawRequester;

pub enum LLMProvider {
    OpenAI(Box<dyn RawRequester + Send>),
    Claude(Box<dyn RawRequester + Send>),
}