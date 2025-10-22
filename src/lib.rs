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
        let request: ChatRequest = ChatRequest::new(
            ModelId::Gpt35Turbo(None),
            vec![
                Message::system_message("You are a helpful assistant."),
                Message::user_message("Hello!"),
            ],
        );

        let expected = r#"
          {
            "model": "gpt-3.5-turbo",
            "messages": [
              {
                "role": "system",
                "content": "You are a helpful assistant."
              },
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
            ModelId::Gpt35Turbo(None),
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
            "model": "gpt-3.5-turbo",
            "messages": [
              {
                "role": "user",
                "content": "What is the weather like in Boston?"
              }
            ],
            "tools": [
              {
                "type": "function",
                "function": {
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
        "created": 1699896916,
        "model": "gpt-3.5-turbo-0613",
        "choices": [
          {
            "index": 0,
            "message": {
              "role": "assistant",
              "content": null,
              "tool_calls": [
                {
                  "id": "call_abc123",
                  "type": "function",
                  "function": {
                    "name": "get_current_weather",
                    "arguments": "{\n\"location\": \"Boston, MA\"\n}"
                  }
                }
              ]
            },
            "logprobs": null,
            "finish_reason": "tool_calls"
          }
        ],
        "usage": {
          "prompt_tokens": 82,
          "completion_tokens": 17,
          "total_tokens": 99
        }
      }
      "#;

        let response: ChatCompletionObject =
            ChatCompletionObject::from_json(&serde_json::from_str(response_raw).unwrap()).unwrap();
        assert_eq!(response.id, "chatcmpl-abc123");
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.created, 1699896916);
        assert_eq!(
            response.model,
            ModelId::Gpt35Turbo(Some("0613".to_string()))
        );
        assert_eq!(response.system_fingerprint, None);
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.usage.prompt_tokens, 82);
        assert_eq!(response.usage.completion_tokens, 17);
        assert_eq!(response.usage.total_tokens, 99);

        assert_eq!(response.choices.len(), 1);
        let assistant_mesg = response.choices[0].message.as_assistant_message().unwrap();
        assert_eq!(assistant_mesg.content, None);
        assert_eq!(assistant_mesg.tool_calls.as_ref().unwrap().len(), 1);
        let tool_call = &assistant_mesg.tool_calls.as_ref().unwrap()[0];
        assert_eq!(tool_call.id, "call_abc123");
        assert_eq!(tool_call.function.name, "get_current_weather");
        assert_eq!(
            tool_call.function.arguments,
            "{\n\"location\": \"Boston, MA\"\n}"
        );
    }

    #[test]
    fn response_from_string() {
        let response_raw = r#"
          {
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-3.5-turbo-0613",
            "system_fingerprint": "fp_44709d6fcb",
            "choices": [{
              "index": 0,
              "message": {
                "role": "assistant",
                "content": "\n\nHello there, how may I assist you today?"
              },
              "logprobs": null,
              "finish_reason": "stop"
            }],
            "usage": {
              "prompt_tokens": 9,
              "completion_tokens": 12,
              "total_tokens": 21
            }
          }
        "#;

        let response: ChatCompletionObject =
            ChatCompletionObject::from_json(&serde_json::from_str(response_raw).unwrap()).unwrap();
        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.created, 1677652288);
        assert_eq!(
            response.model,
            ModelId::Gpt35Turbo(Some("0613".to_string()))
        );
        assert_eq!(response.system_fingerprint.unwrap(), "fp_44709d6fcb");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.usage.prompt_tokens, 9);
        assert_eq!(response.usage.completion_tokens, 12);
        assert_eq!(response.usage.total_tokens, 21);

        assert_eq!(response.choices.len(), 1);
        let ccc = &response.choices[0];
        assert_eq!(ccc.finish_reason, FinishReason::Stop);
        assert_eq!(ccc.index, 0);
        assert_eq!(ccc.logprobs, None);
        let mesg = ccc.message.as_assistant_message().unwrap();
        assert_eq!(
            mesg.content.as_ref().unwrap(),
            "\n\nHello there, how may I assist you today?"
        );
        assert_eq!(mesg.name, None);
        assert_eq!(mesg.tool_calls, None);
    }

    #[test]
    pub fn response_system_fingerprint_can_be_null() {
        let response_raw = r#"
        {
            "id": "chatcmpl-8qXquDiRDSsRnl5ztx0Nz3Ri3nLdC",
            "object": "chat.completion",
            "created": 1707533876,
            "model": "gpt-3.5-turbo-0613",
            "choices": [
              {
                "index": 0,
                "message": {
                  "role": "assistant",
                  "content": "Hi there! How can I assist you today?"
                },
                "logprobs": null,
                "finish_reason": "stop"
              }
            ],
            "usage": {
              "prompt_tokens": 19,
              "completion_tokens": 10,
              "total_tokens": 29
            },
            "system_fingerprint": null
          }
        "#;

        let response: ChatCompletionObject =
            ChatCompletionObject::from_json(&serde_json::from_str(response_raw).unwrap()).unwrap();
        assert_eq!(response.system_fingerprint, None);
    }

    #[test]
    pub fn can_create_chat_completion_choice() {
        let raw_json = json!(
            {
                "index": 0,
                "message": {
                  "role": "assistant",
                  "content": "Hello! How can I assist you today?"
                },
                "logprobs": null,
                "finish_reason": "stop"
            }
        );
        let result = ChatCompletionChoice::from_json(&raw_json).unwrap();
        assert_eq!(result.index, 0);
        assert_eq!(result.logprobs, None);
        assert_eq!(result.finish_reason, FinishReason::Stop);
        let mesg: &AssistantMessage = result.message.as_assistant_message().unwrap();
        assert_eq!(
            mesg.content.as_ref().unwrap(),
            "Hello! How can I assist you today?"
        );
    }

    #[test]
    pub fn can_create_chat_completion_with_tool_choice() {
        let raw_json = json!({
            "index": 0,
            "message": {
              "role": "assistant",
              "content": null,
              "tool_calls": [
                {
                  "id": "call_abc123",
                  "type": "function",
                  "function": {
                    "name": "get_current_weather",
                    "arguments": "{\n\"location\": \"Boston, MA\"\n}"
                  }
                }
              ]
            },
            "logprobs": null,
            "finish_reason": "tool_calls"
            }
        );

        let result = ChatCompletionChoice::from_json(&raw_json).unwrap();
        assert_eq!(result.index, 0);
        assert_eq!(result.logprobs, None);
        assert_eq!(result.finish_reason, FinishReason::ToolCalls);
        let mesg: &AssistantMessage = result.message.as_assistant_message().unwrap();
        assert_eq!(mesg.content, None);
        assert_eq!(mesg.tool_calls.as_ref().unwrap().len(), 1);
        let t = &mesg.tool_calls.as_ref().unwrap()[0];
        assert_eq!(t.id, "call_abc123");
        assert_eq!(t.function.name, "get_current_weather");
        assert_eq!(t.function.arguments, "{\n\"location\": \"Boston, MA\"\n}");
    }

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
    pub fn ping_pong_system_message() {
        do_ping_pong_test::<SystemMessage>()
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
        let mut context = GeneratorContext::new();
        let n_tests = 32;
        for _ in 0..n_tests {
            let original = ModelId::gen(&mut context);
            let json_value = original.to_json();
            let copy = ModelId::from_json(&json_value).unwrap();
            assert_eq!(original, copy);
        }
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
    pub fn system_message_json_has_correct_role() {
        property_test(|m: &SystemMessage| {
            m.to_json()["role"]
                .as_str()
                .map(|r| r == "system")
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
