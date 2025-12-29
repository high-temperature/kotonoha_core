use crate::models::{ChatMessage, ChatRequest, ChatResponse};

use reqwest::Client;

use std::env;
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



fn mock_openai_enabled() -> bool {
    env::var("MOCK_OPENAI").is_ok()
}

fn mock_task_title(input: &str) -> String {
    if let Some(idx) = input.find("タスク") {
        let end = idx + "タスク".len();
        let prefix = &input[..end];
        let start = prefix
            .rfind(|c: char| c.is_whitespace() || ['、', '。', ',', '.', '！', '!', '?', '？'].contains(&c))
            .map(|pos| pos + 1)
            .unwrap_or(0);
        return prefix[start..].to_string();
    }

    input.trim().to_string()
}

pub async fn classify_input(client: &Client, api_key: &str, input: &str) -> Result<String, Box<dyn Error>> {
    if mock_openai_enabled() {
        if input.contains("タスク") || input.contains("やる") || input.contains("完了") {
            return Ok("タスク".to_string());
        }
        return Ok("雑談".to_string());
    }

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

    let status = response.status();
    let text = response.text().await?;
    
    if !status.is_success() {
        return Err(format!("API Error {}: {}", status, text).into());
       }

    let parsed: ChatResponse = serde_json::from_str(&text)?;
    let content = parsed
        .choices
        .get(0)
        .ok_or_else(|| Box::<dyn Error>::from(format!("OpenAI: No choices found in the response: {}", text)))?
        .message
        .content
        .trim()
        .to_lowercase();

    Ok(content)
}

pub async fn classify_task_action(client: &Client, api_key: &str, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    if mock_openai_enabled() {
        if input.contains("完了") {
            return Ok("完了".to_string());
        }
        if input.contains("一覧") {
            return Ok("一覧".to_string());
        }
        if input.contains("追加") || input.contains("覚えて") || input.contains("登録") {
            return Ok("追加".to_string());
        }
        return Ok("なし".to_string());
    }

    let prompt = format!(
        "次のユーザーの発言がタスク操作だとしたら、操作の種類を一語で答えてください。「追加」「完了」「一覧」「なし」のいずれかで返答してください。\n\n入力: {}",
        input
    );

    let req = ChatRequest {
        model: "gpt-3.5-turbo".into(),
        messages: vec![ChatMessage {
            role: "user".into(),
            content: prompt,
        }],
    };

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&req)
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(format!("API Error: ({}):{}", status, body).into());
    }

    let parsed: ChatResponse = serde_json::from_str(&body)?;
    Ok(parsed
        .choices
        .get(0)
        .ok_or_else(|| format!("OpenAI: No choices found in the response: {}", body))?
        .message
        .content
        .trim()
        .to_string()
    )

}


pub fn detect_special_command(input: &str) -> Option<&'static str>{
    if input.contains("タスク一覧") || input.contains("タスク確認"){
        Some("list")
    }else {
        None
    }
}

pub async fn extract_task(client: &Client, api_key: &str, input: &str) -> Result<String, Box<dyn Error>> {
    if mock_openai_enabled() {
        return Ok(mock_task_title(input));
    }

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

    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(format!("API Error: ({}):{}", status, body).into());
    }

    let parsed: ChatResponse = serde_json::from_str(&body)?;
    Ok(parsed
        .choices
        .get(0)
        .ok_or_else(|| format!("OpenAI: No choices found in the response: {}", body))?
        .message
        .content
        .trim()
        .to_string()
    )
}

pub async fn respond_to_chat(client: &Client, api_key: &str, messages: &Vec<ChatMessage>) -> Result<String, Box<dyn Error>> {
    if mock_openai_enabled() {
        return Ok("はい、承知しました。".to_string());
    }

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

    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
            return Err(format!("API Error: ({}):{}", status, text).into());
        }

    let parsed: ChatResponse = serde_json::from_str(&text)?;
    let reply = parsed.choices
        .get(0)
        .map(| choice | choice.message.content.clone())
        .ok_or({
            format!("No choices found in the response: {}", text)
        })?;

    Ok(reply.trim().to_string())
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
