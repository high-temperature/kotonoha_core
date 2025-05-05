use crate::models::Task;
use crate::models::ChatMessage;

use crate::tts;
use chrono::Local;
use tokio::time::{sleep, Duration};
use std::error::Error;

pub fn make_greeting_message(tasks: &[Task]) -> String {
    let pending_count = tasks.iter().filter(|t| !t.done).count();

    if pending_count == 0 {
        "おはようございます。すべてのタスクが完了しています。今日もいい日になりますように。".to_string()
    } else {
        format!("おはようございます。現在 {} 件のタスクがあります。", pending_count)
    }
}
#[cfg(feature = "tts")]
pub async fn greeting(messages:&mut Vec<ChatMessage>) -> Result<(), Box<dyn Error>> {
    let tasks = crate::tasks::load_tasks::<&str>(None);
    let greeting_text = make_greeting_message(&tasks);
    crate::tts::speak(&greeting_text).await?;

    messages.push(ChatMessage {
        role: "assistant".into(),
        content: greeting_text,
    });

    Ok(())
}



#[cfg(feature = "tts")]
pub async fn timer() {
    loop {
        sleep(Duration::from_secs(300)).await;

        let now = Local::now();
        let time_str = now.format("%H時%M分").to_string();
        let message = format!("ただいま、{}です。水分補給と休憩も忘れずに。", time_str);

        let _ = tts::speak(&message).await;
    }
}


#[cfg(test)]
mod tests {
    use serde_json::Map;
    use crate::models::TaskStatus;
    use crate::models::Visibility;
    use super::*;
    use crate::models::Task;

    #[tokio::test]
    async fn test_greeting_with_zero_tasks() {
        let tasks = vec![];
        let message = make_greeting_message(&tasks);
        assert!(message.contains("すべてのタスクが完了"));
    }

    #[tokio::test]
    async fn test_greeting_with_pending_tasks() {
        let tasks = vec![
            Task {
                id: 1,
                title: "テストタスク".into(),
                done: false,
                due_date: None,
                priority: None,
                status: TaskStatus::NotStarted,
                visibility: Visibility::Visible,
                notes: None,
                tags: vec![],
                subtasks: vec![],
                extensions: Map::new(),
            }
        ];
        let message = make_greeting_message(&tasks);
        assert!(message.contains("現在 1 件のタスク"));
    }
}
