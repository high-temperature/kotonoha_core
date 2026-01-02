use kotonoha_core::*;
use crate::{tasks, tts, chat, kotonoha};
use crate::models::ChatMessage;

use dotenvy::dotenv;
use std::env;
use std::io;
use reqwest::Client;

use tokio::sync::mpsc;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();
    
    if let Ok(file_path) = std::env::var("TASK_FILE") {
        kotonoha_core::tasks::set_task_file(&file_path);
    }
    
    if std::env::var("MOCK_TTS").is_ok() {
        tts::enable_mock_mode();
    }
  
    let mock_openai = env::var("MOCK_OPENAI").is_ok();
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_default();
    if api_key.is_empty() && !mock_openai {
        return Err("OPENAI_API_KEY is not set".into());
    }
    
    let client = Client::new();

    let mut messages = vec![
        ChatMessage {
            role: "system".into(),
            content: chat::SYSTEM_PROMPT.into(),
        }
    ];

    kotonoha::greeting(&mut messages).await?;

    //stdin をイベント化

    let (tx, mut rx) = mpsc::channel::<String>(32);
    tokio::task::spawn_blocking(move ||{
        let stdin = io::stdin();
        let mut buf = String::new();
        loop{
            buf.clear();
            if stdin.read_line(&mut buf).ok().filter(|&n| n>0).is_none(){
                break;
            }
            let line = buf.trim().to_string();
            if tx.blocking_send(line).is_err(){
                break;
            }
        }
    });

    // 時報

    let mut time_tick = time::interval(Duration::from_secs(300));

    println!("Kotonoha> こんにちは。ご用件をどうぞ。終了するには 'exit'またはCtrl+C と入力してください。");

    loop{
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nKotonoha> 終了します。またお話ししましょうね！");
                break;
            }

            _ = time_tick.tick() => {
                kotonoha::announce_time_once().await;
            }

            Some(line) = rx.recv() => {
                let user_input = line.trim();
                if user_input.is_empty() {
                    continue;
                }
                if user_input == "exit" {
                    println!("Kotonoha> 終了します。またお話ししましょうね！");
                    break;
                }

                //GPTで分類
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

                    "雑談" => {
                        messages.push(ChatMessage { role: "user".into(), content: user_input.to_string() });
                        let response = chat::respond_to_chat(&client, &api_key, &messages).await?;
                        println!("Kotonoha > {}", response);
                        tts::speak(&response).await?;
                        messages.push(ChatMessage { role: "assistant".into(), content: response });
                    }

                    _ => {
                        tts::speak("分類に失敗しました。もう一度お願いします。").await?;
                    }
                }
            }
        }
    }



    Ok(())
}
