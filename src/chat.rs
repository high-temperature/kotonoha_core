use crate::models::{ChatMessage, ChatRequest, ChatResponse};
use reqwest::Client;
use std::error::Error;

pub async fn classify_input(client: &Client, api_key: &str, input: &str) -> Result<String, Box<dyn Error>> {
    let prompt = format!(
        "以下の文章はユーザからの入力です。この文章が「やるべきこと（ToDo）」に関する指示なら「タスク」、そうでなく会話や質問なら「雑談」とだけ返答してください。\n\n文章：{}",
        input
    );

    let request = ChatRequest {
        model: "gpt-3.5-turbo".into(),
        messages: vec![ChatMessage {
            role: "user".into(),
            content: prompt,
        }],
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await?;

    let text = response.text().await?;
    let parsed: ChatResponse = serde_json::from_str(&text)?;
    Ok(parsed.choices[0].message.content.trim().to_lowercase())
}

pub async fn extract_task(client: &Client, api_key: &str, input: &str) -> Result<String, Box<dyn Error>> {
    let prompt = format!(
        "以下の文から、やるべきタスクがあればタイトルだけを抽出してください。\n文:{}",
        input
    );

    let request = ChatRequest {
        model: "gpt-3.5-turbo".into(),
        messages: vec![ChatMessage {
            role: "user".into(),
            content: prompt,
        }],
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await?;

    let text = response.text().await?;
    let parsed: ChatResponse = serde_json::from_str(&text)?;
    Ok(parsed.choices[0].message.content.trim().to_string())
}

pub async fn respond_to_chat(client: &Client, api_key: &str, input: &str) -> Result<String, Box<dyn Error>> {
    let request = ChatRequest {
        model: "gpt-3.5-turbo".into(),
        messages: vec![ChatMessage {
            role: "user".into(),
            content: input.to_string(),
        }],
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await?;

    let text = response.text().await?;
    let parsed: ChatResponse = serde_json::from_str(&text)?;
    Ok(parsed.choices[0].message.content.trim().to_string())
}


#[cfg(test)]
pub fn make_classification_prompt(input: &str) -> String {
    format!(
        "以下の文章はユーザからの入力です。この文章が「やるべきこと（ToDo）」に関する指示なら「タスク」、そうでなく会話や質問なら「雑談」とだけ返答してください。\n\n文章：{}",
        input
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_classification_prompt_contains_input() {
        let input = "明日までに洗濯";
        let prompt = make_classification_prompt(input);
        assert!(prompt.contains(input));
        assert!(prompt.contains("タスク")); // 安全確認
    }

    #[test]
    fn test_classify_mode_formatting() {
        let result = "タスク".trim().to_lowercase();
        assert_eq!(result, "タスク");
    }
}
