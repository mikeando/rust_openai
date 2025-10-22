pub mod embedding;
pub mod generate;
pub mod json;
pub mod json_ext;
pub mod request;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::generate::{Generatable, GeneratorContext};
    use crate::json::{FromJson, ToJson};
    use crate::types::*;

    use serde_json::json;

    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn request_to_string() {
        let request: ChatRequest =
            ChatRequest::new(ModelId::Gpt5, vec![Message::user_message("Hello!")])
                .with_instructions("You are a helpful assistant.".to_string());

        let expected = r#"
          {
            "model": "gpt-5",
            "instructions": "You are a helpful assistant.",
            "input": [
              {
                "role": "user",
                "content": "Hello!"
              }
            ]
          }
        "#;
        let v: serde_json::Value = serde_json::from_str(expected).unwrap();

        assert_eq!(request.to_json(), v);
    }

    #[test]
    fn request_with_tool_use_to_string() {
        let parameters = json!({
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "The city and state, e.g. San Francisco, CA"
            },
            "unit": {
              "type": "string",
              "enum": ["celsius", "fahrenheit"]
            }
          },
          "required": ["location"]
        });

        let request: ChatRequest = ChatRequest::new(
            ModelId::Gpt5,
            vec![Message::user_message("What is the weather like in Boston?")],
        )
        .with_tool_choice(ToolChoice::Auto)
        .with_tools(vec![Tool {
            description: Some("Get the current weather in a given location".to_string()),
            name: "get_current_weather".to_string(),
            parameters: Some(JSONSchema::from_json(&parameters).unwrap()),
        }]);

        let expected = r#"
                    {
                        "model": "gpt-5",
                        "input": [
                            {
                                "role": "user",
                                "content": "What is the weather like in Boston?"
                            }
                        ],
                        "tools": [
                            {
                                "type": "function",
                                "name": "get_current_weather",
                                "description": "Get the current weather in a given location",
                                "parameters": {
                                    "type": "object",
                                    "properties": {
                                        "location": {
                                            "type": "string",
                                            "description": "The city and state, e.g. San Francisco, CA"
                                        },
                                        "unit": {
                                            "type": "string",
                                            "enum": ["celsius", "fahrenheit"]
                                        }
                                    },
                                    "required": ["location"]
                                }
                            }
                        ],
                        "tool_choice": "auto"
                    }
                "#;

        let v: serde_json::Value = serde_json::from_str(expected).unwrap();

        assert_eq!(request.to_json(), v);
    }

    #[test]
    fn response_from_string_with_tools() {
        let response_raw = r#"
        {
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "created_at": 1699896916,
            "model": "gpt-5",
            "output": [
                {
                    "id": "fc_abc123",
                    "type": "function_call",
                    "status": "completed",
                    "arguments": "{\n\"location\": \"Boston, MA\"\n}",
                    "call_id": "call_abc123",
                    "name": "get_current_weather"
                }
            ],
            "usage": {
                "input_tokens": 82,
                "input_tokens_details": { "cached_tokens": 0 },
                "output_tokens": 17,
                "output_tokens_details": { "reasoning_tokens": 5 },
                "total_tokens": 99
            }
        }
        "#;

        let response: ChatCompletionObject =
            ChatCompletionObject::from_json(&serde_json::from_str(response_raw).unwrap()).unwrap();
        assert_eq!(response.id, "chatcmpl-abc123");
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.created_at, 1699896916);
        assert_eq!(response.model.name(), "gpt-5");
        assert_eq!(response.system_fingerprint, None);
        assert_eq!(response.output.len(), 1);
        assert_eq!(response.usage.input_tokens, 82);
        assert_eq!(response.usage.output_tokens, 17);
        assert_eq!(response.usage.total_tokens, 99);
        let function_call = &response.output[0];
        assert_eq!(function_call.id.as_ref().unwrap(), "fc_abc123");
        assert_eq!(function_call.output_type.as_ref().unwrap(), "function_call");
        assert_eq!(function_call.status.as_ref().unwrap(), "completed");
        assert_eq!(function_call.name.as_ref().unwrap(), "get_current_weather");
        assert_eq!(
            function_call.arguments.as_ref().unwrap(),
            "{\n\"location\": \"Boston, MA\"\n}"
        );
    }

    #[test]
    fn response_from_string() {
        let response_raw = r#"
        {
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created_at": 1677652288,
            "model": "gpt-5",
            "system_fingerprint": "fp_44709d6fcb",
            "output": [{
                "id": "msg_123",
                "type": "message",
                "content": "\n\nHello there, how may I assist you today?"
            }],
            "usage": {
                "input_tokens": 9,
                "input_tokens_details": { "cached_tokens": 0 },
                "output_tokens": 12,
                "output_tokens_details": { "reasoning_tokens": 2 },
                "total_tokens": 21
            }
        }
        "#;

        let response: ChatCompletionObject =
            ChatCompletionObject::from_json(&serde_json::from_str(response_raw).unwrap()).unwrap();
        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.created_at, 1677652288);
        assert_eq!(response.model.name(), "gpt-5");
        assert_eq!(response.system_fingerprint.unwrap(), "fp_44709d6fcb");
        assert_eq!(response.output.len(), 1);
        assert_eq!(response.usage.input_tokens, 9);
        assert_eq!(response.usage.output_tokens, 12);
        assert_eq!(response.usage.total_tokens, 21);
        let msg = &response.output[0];
        assert_eq!(msg.id.as_ref().unwrap(), "msg_123");
        assert_eq!(msg.output_type.as_ref().unwrap(), "message");
        assert_eq!(
            msg.content.as_ref().unwrap(),
            "\n\nHello there, how may I assist you today?"
        );
    }

    #[test]
    pub fn response_system_fingerprint_can_be_null() {
        let response_raw = r#"
        {
            "id": "chatcmpl-8qXquDiRDSsRnl5ztx0Nz3Ri3nLdC",
            "object": "chat.completion",
            "created_at": 1707533876,
            "model": "gpt-5",
            "output": [{
                "id": "msg_8qXquDiRDSsRnl5ztx0Nz3Ri3nLdC",
                "type": "message",
                "content": "Hi there! How can I assist you today?"
            }],
            "usage": {
                "input_tokens": 19,
                "input_tokens_details": { "cached_tokens": 0 },
                "output_tokens": 10,
                "output_tokens_details": { "reasoning_tokens": 3 },
                "total_tokens": 29
            },
            "system_fingerprint": null
        }
        "#;

        let response: ChatCompletionObject =
            ChatCompletionObject::from_json(&serde_json::from_str(response_raw).unwrap()).unwrap();
        assert_eq!(response.system_fingerprint, None);
        let msg = &response.output[0];
        assert_eq!(
            msg.id.as_ref().unwrap(),
            "msg_8qXquDiRDSsRnl5ztx0Nz3Ri3nLdC"
        );
        assert_eq!(msg.output_type.as_ref().unwrap(), "message");
        assert_eq!(
            msg.content.as_ref().unwrap(),
            "Hi there! How can I assist you today?"
        );
    }

    // ...existing code...

    #[test]
    pub fn generate_string_is_unique() {
        let mut context = GeneratorContext::new();
        let s1 = String::gen(&mut context);
        let s2 = String::gen(&mut context);
        assert_ne!(s1, s2);
    }

    pub fn do_ping_pong_test<T>()
    where
        T: Generatable + ToJson + FromJson + std::fmt::Debug + std::cmp::PartialEq,
    {
        let mut context = GeneratorContext::new();
        let n_tests = 32;
        for _ in 0..n_tests {
            let original = T::gen(&mut context);
            let json_value = original.to_json();
            let copy = T::from_json(&json_value).unwrap();
            assert_eq!(original, copy);
        }
    }

    pub fn property_test<T, F>(f: F)
    where
        T: Generatable,
        F: Fn(&T) -> bool,
    {
        let mut context = GeneratorContext::new();
        let n_tests = 32;
        for _ in 0..n_tests {
            let v = T::gen(&mut context);
            assert!(f(&v));
        }
    }

    #[test]
    pub fn ping_pong_tool_call() {
        do_ping_pong_test::<ToolCall>();
    }

    #[test]
    pub fn ping_pong_tool_function() {
        do_ping_pong_test::<ToolFunction>()
    }

    #[test]
    pub fn ping_pong_message() {
        do_ping_pong_test::<Message>()
    }

    #[test]
    pub fn ping_pong_user_message() {
        do_ping_pong_test::<UserMessage>()
    }

    #[test]
    pub fn ping_pong_assistant_message() {
        do_ping_pong_test::<AssistantMessage>()
    }

    #[test]
    pub fn ping_pong_tool_message() {
        do_ping_pong_test::<ToolMessage>()
    }

    #[test]
    pub fn ping_pong_chat_completion_object() {
        do_ping_pong_test::<ChatCompletionObject>()
    }

    #[test]
    pub fn ping_pong_chat_completion_choice() {
        do_ping_pong_test::<ChatCompletionChoice>()
    }

    #[test]
    pub fn ping_pong_model_id() {
        do_ping_pong_test::<ModelId>()
    }

    #[test]
    pub fn ping_pong_usage_stats() {
        do_ping_pong_test::<UsageStats>()
    }

    #[test]
    pub fn ping_pong_finish_reason() {
        do_ping_pong_test::<FinishReason>()
    }

    #[test]
    pub fn ping_pong_response_format() {
        do_ping_pong_test::<ResponseFormat>()
    }

    #[test]
    pub fn ping_pong_chat_request() {
        do_ping_pong_test::<ChatRequest>()
    }

    #[test]
    pub fn ping_pong_logit_bias() {
        do_ping_pong_test::<LogitBias>()
    }

    #[test]
    pub fn ping_pong_tool() {
        do_ping_pong_test::<Tool>()
    }

    #[test]
    pub fn assistant_message_json_has_correct_role() {
        property_test(|a: &AssistantMessage| {
            a.to_json()["role"]
                .as_str()
                .map(|r| r == "assistant")
                .unwrap_or(false)
        })
    }

    #[test]
    pub fn user_message_json_has_correct_role() {
        property_test(|m: &UserMessage| {
            m.to_json()["role"]
                .as_str()
                .map(|r| r == "user")
                .unwrap_or(false)
        })
    }

    #[test]
    pub fn tool_message_json_has_correct_role() {
        property_test(|m: &ToolMessage| {
            m.to_json()["role"]
                .as_str()
                .map(|r| r == "tool")
                .unwrap_or(false)
        })
    }
}
