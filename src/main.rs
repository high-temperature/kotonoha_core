use kotonoha_core::*;
use tts;
use models::ChatMessage;

use dotenvy::dotenv;
use std::env;
use std::io::{self, Write};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // MOCK_TTS 環境変数がセットされていたらモックモードにする
    if std::env::var("MOCK_TTS").is_ok() {
        tts::enable_mock_mode();
    }

    let api_key = env::var("OPENAI_API_KEY")?;
    let client = Client::new();
    let _messages:Vec<ChatMessage> = vec![];

    // Kotonohaが定期的にしゃべる
    tokio::spawn(async {
        kotonoha::timer().await;
    });

    let task_file = env::var("TASK_FILE").unwrap_or_else(|_| "tasks.json".to_string());
    tasks::set_task_file(&task_file);

    kotonoha::greeting().await?;


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

        let mode = chat::classify_input(&client, &api_key, user_input).await?;
        match mode.as_str() {
            "タスク" => {
                let task = chat::extract_task(&client, &api_key, user_input).await?;
                if task.is_empty() || task == "なし" {
                    tts::speak("タスクは見つかりませんでした。").await?;
                } else {
                    tasks::add_task(&task).await;
                }
            },
            "雑談" => {
                let response = chat::respond_to_chat(&client, &api_key, user_input).await?;
                    println!("Kotonoha > {}", response);
                    tts::speak(&response).await?;
            },
            _ => {
            tts::speak("分類に失敗しました。").await?;
            }
        }       
    }

    Ok(())
}