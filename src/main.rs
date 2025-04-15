mod tasks;
mod tts;


use dotenvy::dotenv;
use std::env;
use std::io::{self, Write};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use tokio::time::{sleep, Duration};
use chrono::Local;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY")?;
    let client = Client::new();
    let _messages:Vec<ChatMessage> = vec![];

    // Kotonohaが定期的にしゃべる
    tokio::spawn(async {
        kotonoha_timer().await;
    });


    kotonoha_greeting().await?;


    loop {
        print!("あなた > ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();
    
        if user_input == "exit" {
            break;
        }
    
        // ✅ 手動コマンド処理
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
    
        // 🧠 雑談 or タスク分類
        let classification_prompt = format!(
            "以下の文章はユーザからの入力です。この文章が「やるべきこと（ToDo）」に関する指示なら「タスク」、そうでなく会話や質問なら「雑談」とだけ返答してください。\n\n文章：{}",
            user_input
        );
    
        let classification_request = ChatRequest {
            model: "gpt-3.5-turbo".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: classification_prompt,
            }],
        };
    
        let classification_response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&api_key)
            .json(&classification_request)
            .send()
            .await?;
    
        let classification_text = classification_response.text().await?;
        let classification_resp: ChatResponse = serde_json::from_str(&classification_text)?;
    
        let mode = classification_resp.choices[0].message.content.trim().to_lowercase();
    
        match mode.as_str() {
            "タスク" => {
                // ✨ タスク抽出プロンプトを使って再送信
                let extraction_prompt = format!(
                    "以下の文から、やるべきタスクがあればタイトルだけを抽出してください。\n文:{}",
                    user_input
                );
    
                let task_request = ChatRequest {
                    model: "gpt-3.5-turbo".into(),
                    messages: vec![ChatMessage {
                        role: "user".into(),
                        content: extraction_prompt,
                    }],
                };
    
                let task_response = client
                    .post("https://api.openai.com/v1/chat/completions")
                    .bearer_auth(&api_key)
                    .json(&task_request)
                    .send()
                    .await?;
    
                let task_text = task_response.text().await?;
                let task_result: ChatResponse = serde_json::from_str(&task_text)?;
                let reply = task_result.choices[0].message.content.trim().to_string();
    
                if reply.is_empty() || reply == "なし" {
                    println!("Kotonoha > タスクは見つかりませんでした。");
                    tts::speak("タスクは見つかりませんでした。").await?;
                } else {
                    tasks::add_task(&reply).await;
                }
            }
    
            "雑談" => {
                // 通常の雑談モードで返答
                let chat_request = ChatRequest {
                    model: "gpt-3.5-turbo".into(),
                    messages: vec![ChatMessage {
                        role: "user".into(),
                        content: user_input.to_string(),
                    }],
                };
    
                let chat_response = client
                    .post("https://api.openai.com/v1/chat/completions")
                    .bearer_auth(&api_key)
                    .json(&chat_request)
                    .send()
                    .await?;
    
                let chat_text = chat_response.text().await?;
                let chat_result: ChatResponse = serde_json::from_str(&chat_text)?;
                let reply = chat_result.choices[0].message.content.trim();
    
                println!("Kotonoha > {}", reply);
                tts::speak(reply).await?;
            }
    
            _ => {
                // 失敗時のフォールバック
                println!("Kotonoha > ごめんなさい、分類に失敗しました。");
                tts::speak("ごめんなさい、よくわかりませんでした。").await?;
            }
        }
    }
    

    Ok(())
}

// Kotonoha が起動時に挨拶をしてタスクを確認する
async fn kotonoha_greeting() -> Result<(), Box<dyn std::error::Error>> {
    let tasks = tasks::load_tasks();
    let pending_count = tasks.iter().filter(|task| !task.done).count();

    let greeting_message = if pending_count == 0 {
        "おはようございます。現在登録されているすべてのタスクが完了しています。".to_string()
    } else {
        format!("おはようございます。現在 {} 件のタスクがあります。", pending_count)
    };

    tts::speak(&greeting_message).await?;
        
    Ok(())
}

//Kotonohaが定期的にしゃべる
async fn kotonoha_timer(){
    loop{
        sleep(Duration::from_secs(300)).await;

        let now = Local::now();
        let time_str = now.format("%H時%M分").to_string();
        let timer_message = format!("ただいま、{}です。姿勢を正して頑張りましょう。", time_str);

        let _ = tts::speak(&timer_message).await;
    }
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
