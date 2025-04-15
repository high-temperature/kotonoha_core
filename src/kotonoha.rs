use crate::tts;
use crate::tasks;
use chrono::Local;
use tokio::time::{sleep, Duration};
use std::error::Error;

pub async fn greeting() -> Result<(), Box<dyn Error>> {
    let tasks = tasks::load_tasks();
    let pending_count = tasks.iter().filter(|task| !task.done).count();

    let greeting_message = if pending_count == 0 {
        "おはようございます。すべてのタスクが完了しています。今日もいい日になりますように。".to_string()
    } else {
        format!("おはようございます。現在 {} 件のタスクがあります。", pending_count)
    };

    tts::speak(&greeting_message).await?;
    Ok(())
}

pub async fn timer() {
    loop {
        sleep(Duration::from_secs(300)).await;

        let now = Local::now();
        let time_str = now.format("%H時%M分").to_string();
        let message = format!("ただいま、{}です。水分補給と休憩も忘れずに。", time_str);

        let _ = tts::speak(&message).await;
    }
}
