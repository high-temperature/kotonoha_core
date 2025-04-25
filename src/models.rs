use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub done: bool,
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"
        {
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "こんにちは"
                    }
                }
            ]
        }
        "#;

        let parsed: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.choices[0].message.content, "こんにちは");
    }
}
