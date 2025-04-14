mod tasks;
mod tts;

use dotenvy::dotenv;
use std::env;
use std::io::{self, Write};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = Client::new();
    let mut messages = vec![];

    println!("Kotonohaとお話ししましょう。終了したいときは 'exit' と入力してください。");

    loop {
        print!("あなた > ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input == "exit" {
            break;
        }

        // ✅ タスクコマンドの処理
        if user_input.starts_with("todo ") {
            let task = user_input.strip_prefix("todo ").unwrap();
            tasks::add_task(task).await;
            continue;
        }

        if user_input == "list" {
            tasks::list_tasks().await;
            continue;
        }

        if user_input.starts_with("done ") {
            if let Ok(id) = user_input.strip_prefix("done ").unwrap().parse::<u32>() {
                tasks::mark_done(id).await;
            } else {
                println!("⚠️ IDが正しくありません。例: done 1");
            }
            continue;
        }

        // 💬 ChatGPT に送信
        messages.push(ChatMessage {
            role: "user".into(),
            content: user_input.to_string(),
        });

        let request_body = ChatRequest {
            model: "gpt-3.5-turbo".into(),
            messages: messages.clone(),
        };

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&api_key)
            .json(&request_body)
            .send()
            .await?;

        let text = response.text().await?;
        let resp_json: Result<ChatResponse, _> = serde_json::from_str(&text);

        if let Ok(resp) = resp_json {
            let reply = &resp.choices[0].message.content;
            println!("Kotonoha > {}", reply);

            // 🗣 TTSで返答を再生
            tts::speak(reply).await?;

            messages.push(ChatMessage {
                role: "assistant".into(),
                content: reply.clone(),
            });
        } else {
            println!("❌ エラー応答: {}", text);
        }
    }

    Ok(())
}

#[derive(Serialize, Clone)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Clone, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}
