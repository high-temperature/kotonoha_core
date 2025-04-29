use kotonoha_core::*;
use tts;
use chat;
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

    // Kotonohaが定期的にしゃべる
    tokio::spawn(async {
        kotonoha::timer().await;
    });

    let task_file = env::var("TASK_FILE").unwrap_or_else(|_| "tasks.json".to_string());
    tasks::set_task_file(&task_file);

    let mut messages = vec![
        ChatMessage {
            role: "system".into(),
            content: chat::SYSTEM_PROMPT.into(),
        },
        ChatMessage {
            role: "assistant".into(),
            content: chat::FIRST_GREETING.into(),
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

        if let Some(command) = chat::detect_special_command(user_input) {
            match command {
                "list" => {
                    tasks::list_tasks().await;
                    continue;
                },
                _ => {}
            }
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
            "雑談" => {// タスク情報をサマリーする
                let task_summary = tasks::summarize_tasks_for_prompt();
                
                // ユーザー発言＋タスク情報をまとめたメッセージをpush
                messages.push(ChatMessage {
                    role: "user".into(),
                    content: format!(
                        "【現在のタスク状況】\n{}\n\n【ユーザー発言】\n{}",
                        task_summary,
                        user_input
                    ),
                });
                
                //  messages全体をChatGPTに渡す
                let response = chat::respond_to_chat(&client, &api_key, &messages).await?;
                
                // Assistantの応答も履歴にpush
                messages.push(ChatMessage {
                    role: "assistant".into(),
                    content: response.clone(),
                });
                
                //  Kotonohaが返事する
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