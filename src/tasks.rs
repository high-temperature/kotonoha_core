use crate::tts;
use serde::{Serialize, Deserialize};
use std::fs::{
    File,
    OpenOptions,
};
use std::io::{BufReader,BufWriter};
use std::path::Path;

const TASK_FILE: &str = "tasks.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub done: bool,
}   

pub fn load_tasks() -> Vec<Task> {
    if !Path::new(TASK_FILE).exists() {
        return vec![];
    }

    let file = File::open(TASK_FILE).expect("Failed to open tasks file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_else(|_| vec![])

}

pub fn save_tasks(tasks: &[Task]){
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(TASK_FILE)
        .expect("Failed to open tasks file for writing");

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, tasks).expect("Failed to write tasks to file");
}

pub async fn add_task(title: &str) {
    let mut tasks = load_tasks();
    let new_id = tasks.len() as u32 + 1;
    tasks.push(Task {
        id: new_id,
        title: title.to_string(),
        done: false,
    });
    save_tasks(&tasks);

    println!("Kotonoha > タスク「{}」を登録しました。", title);
    let response = format!("タスクを「{}」を登録しました。", title);
    let _ = tts::speak(&response).await;
}
pub async fn list_tasks() {
    let tasks = load_tasks();

    if tasks.is_empty() {
        println!("📋 登録されたタスクはありません。");
        let _ = crate::tts::speak("現在のタスクはすべて完了しています。").await;
    } else {
        println!("📋 現在のタスク一覧:");
        let mut spoken = format!("現在のタスクは {} 件あります。", tasks.len());

        for (i, task) in tasks.iter().enumerate() {
            println!(
                "{}: {} [{}]",
                task.id,
                task.title,
                if task.done { "✅" } else { "　" }
            );

            // ✅ タスクが未完了なら読み上げ内容に追加
            if !task.done {
                spoken.push_str(&format!(" {}つ目、{}。", i + 1, task.title));
            }
        }

        // 🗣 声で読み上げる
        let _ = crate::tts::speak(&spoken).await;
    }
}


pub async fn mark_done(task_id:u32){
    let mut tasks = load_tasks();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id){
        task.done = true;
        println!("タスク {} を完了にしました。", task.title);

        let response = format!("タスク「{}」を完了にしました。", task.title);
        let _ = tts::speak(&response).await;
    }else{
        println!("タスク {} は見つかりませんでした。", task_id);

        let response = format!("タスク {} は見つかりませんでした。", task_id);
        let _ = tts::speak(&response).await;

    }
    save_tasks(&tasks);

}