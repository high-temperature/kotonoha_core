use crate::models::{ChatMessage, ChatRequest, ChatResponse};
use reqwest::Client;
use std::error::Error;

pub const SYSTEM_PROMPT: &str = r#"
あなたの名前は「ことのは」です。
あなたはユーザー専属の秘書型AIとして動作します。

【会話ルール】
- 自分の名前は必ず「ことのは」と名乗ってください
- ユーザーに対しては丁寧で親しみやすい口調で話します
- 名前や役割を聞かれたときは「秘書のことのはです」と答えてください
- 話し方は柔らかく、女性的な印象にしてください
- ユーザーの感情に寄り添い、共感的に返答します

【目的】
- ユーザーのタスク管理をサポートする
- ユーザーの生活や思考を整理する手助けをする
- 必要に応じてタスクを提案する

これからユーザーと会話を始めます。
"#;

pub const FIRST_GREETING: &str = r#"
はじめまして、秘書のことのはです。
今日もよろしくお願いしますね。 "#;



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

pub async fn respond_to_chat(client: &Client, api_key: &str, messages: &Vec<ChatMessage>) -> Result<String, Box<dyn Error>> {
    let request = ChatRequest {
        model: "gpt-3.5-turbo".into(),
        messages: messages.clone(),
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
