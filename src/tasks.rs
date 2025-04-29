use crate::tts;
use crate::models::Task;

use std::fs::{File, OpenOptions};
use std::io::{BufReader,BufWriter};
use std::path::Path;
use std::sync::OnceLock;

static TASK_FILE: OnceLock<String> = OnceLock::new();

pub fn set_task_file(file: &str) {
    let _ = TASK_FILE.set(file.to_string());
}

fn get_task_file() ->&'static str {
    TASK_FILE.get().map(|s| s.as_str()).unwrap_or("tasks.json")
}

pub fn load_tasks_with_file(file:&str)->Vec<Task>{
    if !Path::new(file).exists() {
        return vec![];
    }

    let file = File::open(file).expect("Failed to open tasks file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_else(|_| vec![])

}
pub fn save_tasks_with_file(file:&str,tasks:&[Task]){
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file)
        .expect("Failed to open tasks file for writing");

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, tasks).expect("Failed to write tasks to file");
}

pub fn load_tasks() -> Vec<Task> {
    let file = get_task_file();
    load_tasks_with_file(file)
}

pub fn save_tasks(tasks: &[Task]){
    let file = get_task_file();
    save_tasks_with_file(file, tasks);
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

/// タスク一覧をまとめた文字列を返す
pub fn summarize_tasks_for_prompt() -> String {
    let tasks = load_tasks();
    if tasks.is_empty() {
        "現在、登録されているタスクはありません。".to_string()
    } else {
        let list = tasks
            .iter()
            .filter(|t| !t.done)  // 未完了タスクだけ
            .map(|t| format!("・{}", t.title))
            .collect::<Vec<_>>()
            .join("\n");

        format!("現在の未完了タスク一覧:\n{}", list)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILE_ADD: &str = "tasks_test_add.json";
    const TEST_FILE_DONE: &str = "tasks_test_done.json";
    
    #[test]
    fn test_add_and_load_tasks() {
        let _ = std::fs::remove_file(TEST_FILE_ADD);
    
        let mut tasks = load_tasks_with_file(TEST_FILE_ADD);
        tasks.push(Task {
            id: 1,
            title: "テストタスク".to_string(),
            done: false,
        });
        save_tasks_with_file(TEST_FILE_ADD, &tasks);
    
        let loaded = load_tasks_with_file(TEST_FILE_ADD);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "テストタスク");
        assert!(!loaded[0].done);
    }
    

    #[test]
fn test_mark_done_updates_task() {
    let _ = std::fs::remove_file(TEST_FILE_DONE);

    let tasks = vec![Task {
        id: 1,
        title: "完了チェック".to_string(),
        done: false,
    }];
    save_tasks_with_file(TEST_FILE_DONE, &tasks);

    let mut loaded = load_tasks_with_file(TEST_FILE_DONE);
    if let Some(task) = loaded.iter_mut().find(|t| t.id == 1) {
        task.done = true;
    } else {
        panic!("タスクが見つかりませんでした");
    }
    save_tasks_with_file(TEST_FILE_DONE, &loaded);

    let updated = load_tasks_with_file(TEST_FILE_DONE);
    let updated_task = updated
        .iter()
        .find(|t| t.id == 1)
        .expect("更新後のタスクが見つかりません");
    assert!(updated_task.done);
}

#[test]
fn test_add_multiple_tasks_and_order() {
    let file = get_task_file();
    let _ = std::fs::remove_file(file);
    let mut tasks = vec![];

    tasks.push(Task { id: 1, title: "一件目".to_string(), done: false });
    tasks.push(Task { id: 2, title: "二件目".to_string(), done: false });

    save_tasks_with_file(file, &tasks);
    let loaded = load_tasks_with_file(file);

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].title, "一件目");
    assert_eq!(loaded[1].title, "二件目");
}

    
}
