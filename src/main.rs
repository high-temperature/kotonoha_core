use kotonoha_core::*;
use crate::{tasks, tts, chat, kotonoha};
use crate::models::ChatMessage;

use kotonoha_core::speech::SpeechQueue;

use dotenvy::dotenv;
use std::env;
use std::io;
use reqwest::Client;

use tokio::sync::mpsc;
use tokio::time::{self, Duration, Instant};

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();
    
    if let Ok(file_path) = std::env::var("TASK_FILE") {
        kotonoha_core::tasks::set_task_file(&file_path);
    }
    
    if std::env::var("MOCK_TTS").is_ok() {
        tts::enable_mock_mode();
    }

    let speech = SpeechQueue::spawn(
        Duration::from_secs(600),   // 独り言クールダウン
        Duration::from_secs(300),   // ユーザー操作後抑制時間
    );
  
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

    // ★期限チェック（まずは1時間に1回）
    let mut due_tick = time::interval(Duration::from_secs(10));

    // ★同じタスクを連呼しない（id -> 最後に通知した時刻）
    let mut last_notified: HashMap<u32, Instant> = HashMap::new();
    let notify_cooldown = Duration::from_secs(6 * 3600); // 同一タスクは6時間おき

    // 期限通知の「今やる？」待ち（タスクIDと期限日を保持）
    let mut pending_due: Option<(u32, chrono::NaiveDate)> = None;


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

            _ = due_tick.tick() => {

                if pending_due.is_some() {
                    // 「今やる？」待ち中は新規通知しない
                    continue;
                }
                // 例：3日以内の期限を通知
                let due_tasks = tasks::find_due_within_days(3);

                for t in due_tasks {
                    let now = Instant::now();
                    let should_notify = match last_notified.get(&t.id) {
                        Some(prev) => now.duration_since(*prev) >= notify_cooldown,
                        None => true,
                    };

                    if should_notify {
                        last_notified.insert(t.id, now);
                        // due_date は Option<NaiveDate> なので unwrap は安全（find_due_within_days が Some のみ返す想定）
                        let due = t.due_date.unwrap();
                        let msg = format!("期限が近いタスクがあります：{}（期限: {}）。いまやりますか？(yes/no)", t.title, due);
                        speech.say_alert(msg).await;
                        pending_due = Some((t.id, due));

                        // 今回は1件だけ通知して、あとは次のtickまで待つ
                        break;
                    }
                }
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
                     // ★期限の「いまやる？」待ちがあるなら、それを最優先で処理
                    let input = user_input.trim().to_lowercase();

                    if let Some((task_id, _due)) = pending_due.clone() {
                        let yes = matches!(input.as_str(), "yes" | "y" | "はい" | "やる" | "やります" | "今やる");
                        let no  = matches!(input.as_str(), "no"  | "n" | "いいえ" | "やらない" | "やりません" | "あとで");

                        if yes {
                           if let Some(title) = tasks::get_task_title(task_id) {
                                speech
                                  .say_user(format!("了解です。『{}』を今やりましょう。", title))
                                    .await;
                            } else {
                                speech.say_user("了解です。今やりましょう。".to_string()).await;
                            }

                            pending_due = None;
                            continue;
                        }

                        if no {
                            speech.say_user("わかりました。あとでリマインドしますね。".to_string()).await;
                            pending_due = None;
                            continue;
                        }

                        speech.say_alert("「yes」か「no」でお答えください。".to_string()).await;
                        continue;
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
                                    speech.say_alert("追加するタスクが見つかりませんでした。もう一度お願いします。").await;
                                } else {
                                    tasks::add_task(&task).await;
                                }
                            }
                            "完了" => {
                                if let Some(task_id) = tasks::find_task_id_by_similarity(user_input, 0.85) {
                                    tasks::mark_done(task_id).await;
                                } else {
                                    speech.say_alert("完了タスクが見つかりませんでした。").await;
                                }
                            }
                            "一覧" => {
                                tasks::list_tasks().await;
                            }
                            "なし" | _ => {
                                speech.say_alert("特別な操作はありません。").await;
                            }
                        }
                    }

                    "雑談" => {
                        messages.push(ChatMessage { role: "user".into(), content: user_input.to_string() });
                        let response = chat::respond_to_chat(&client, &api_key, &messages).await?;
                        println!("Kotonoha > {}", response);
                        speech.say_user(&response).await;
                        messages.push(ChatMessage { role: "assistant".into(), content: response });
                    }

                    _ => {
                        speech.say_alert("分類に失敗しました。もう一度お願いします。").await;
                    }
                }
            }
        }
    }



    Ok(())
}
