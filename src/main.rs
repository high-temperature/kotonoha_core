use kotonoha_core::*;
use crate::{tasks, tts, chat, kotonoha};
use crate::models::ChatMessage;

use rand::Rng;

use dotenvy::dotenv;
use std::env;
use std::io::{self, Write};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    if std::env::var("MOCK_TTS").is_ok() {
        tts::enable_mock_mode();
    }

    let api_key = env::var("OPENAI_API_KEY")?;
    let client = Client::new();

    // Kotonoha定時発話
    tokio::spawn(async {
        kotonoha::timer().await;
    });

    let task_file = env::var("TASK_FILE").unwrap_or_else(|_| "tasks.json".to_string());
    tasks::set_task_file(&task_file);

    let mut messages = vec![
        ChatMessage {
            role: "system".into(),
            content: chat::SYSTEM_PROMPT.into(),
        }
    ];

    kotonoha::greeting(&mut messages).await?;

    loop {
        print!("あなた > ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input == "exit" {
            break;
        }

        // 直接コマンド
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

        // GPTで分類（タスク or 雑談）
        let mode = chat::classify_input(&client, &api_key, user_input).await?;

        match mode.as_str() {
            "タスク" => {
                let intent = chat::classify_task_action(&client, &api_key, user_input).await?;
                match intent.as_str() {
                    "追加" => {
                        let task = chat::extract_task(&client, &api_key, user_input).await?;
                        if task.is_empty() || task == "なし" {
                            tts::speak("追加タスクは見つかりませんでした。").await?;
                        } else {
                            tasks::add_task(&task).await;
                        }
                    }
                    "完了" => {
                        if let Some(task_id) = tasks::find_task_id_by_similarity(user_input, 0.85) {
                            tasks::mark_done(task_id).await;
                        } else {
                            tts::speak("完了タスクが見つかりませんでした。").await?;
                        }
                    }
                    "一覧" => {
                        tasks::list_tasks().await;
                    }
                    "なし" | _ => {
                        tts::speak("特別な操作はありません。").await?;
                    }
                }
            }
        ,
            "雑談" => {
                messages.push(ChatMessage {
                    role: "user".into(),
                    content: user_input.to_string(),
                });

                let response = chat::respond_to_chat(&client, &api_key, &messages).await?;

                println!("Kotonoha > {}", response);
                tts::speak(&response).await?;

                messages.push(ChatMessage {
                    role: "assistant".into(),
                    content: response,
                });
            },
            _ => {
                tts::speak("分類に失敗しました。もう一度お願いします。").await?;
            }
        }
    }

    Ok(())
}
