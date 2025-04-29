use crate::tts;
use crate::models::{Task, TaskStatus, Visibility};

use std::fs::{File, OpenOptions};
use std::io::{BufReader,BufWriter};
use std::path::Path;
use std::sync::OnceLock;

use serde_json::Map;

use strsim::jaro_winkler;

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
    let new_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;

    let new_task = Task {
        id: new_id,
        title: title.to_string(),
        done: false,
        due_date: None,
        priority: None,
        status: TaskStatus::NotStarted,
        visibility: Visibility::Visible,
        notes: None,
        tags: vec![],
        subtasks: vec![],
        extensions: Map::new(),
    };

    tasks.push(new_task);
    save_tasks(&tasks);

    println!("Kotonoha > タスク「{}」を登録しました。", title);
    let response = format!("タスクを「{}」を登録しました。", title);
    let _ = tts::speak(&response).await;
}
pub async fn list_tasks() {
    let tasks = load_tasks();

    if tasks.is_empty() {
        println!("登録されたタスクはありません。");
        let _ = crate::tts::speak("現在のタスクはすべて完了しています。").await;
    } else {
        println!("現在のタスク一覧:");
        let _spoken = format!("現在のタスクは {} 件あります。", tasks.len());

        for task in tasks{
            display_tasks(&task, 0);
        }
    }
}

fn display_tasks(task: &Task, indent: usize) {
    let prefix = " ".repeat(indent * 2);
    println!(
        "{}{}: {} [{}]",
        prefix,
        task.id,
        task.title,
        if task.done { "✅" } else { "　" }
    );
    for subtask in &task.subtasks {
        display_tasks(subtask, indent + 1);
    }
}

pub fn find_task_id_by_similarity(input: &str, threshold: f64) -> Option<u32> {
    let tasks = load_tasks();
    let mut best_match = None;
    let mut best_score = 0.0; // 初期スコアを0.0にする

    println!("🔍 入力: \"{}\"", input);

    for task in &tasks {
        if let Some((id, score)) = find_best_match(task, input) {
            println!("📝 タスク \"{}\" のスコア: {:.3}", task.title, score);

            if score > best_score {
                best_match = Some(id);
                best_score = score;
            }
        }
    }

    // 閾値を超えているかチェック
    if best_score >= threshold {
        println!("✅ ベストマッチ: タスクID {} (スコア {:.3})", best_match.unwrap(), best_score);
        best_match
    } else {
        println!("❌ 適合するタスクはありません（最高スコア {:.3}）", best_score);
        None
    }
}

fn find_best_match(task: &Task, input: &str) -> Option<(u32, f64)> {
    let score = jaro_winkler(&task.title.to_lowercase(), &input.to_lowercase());

    if !task.done {
        return Some((task.id, score));
    }

    for sub in &task.subtasks {
        if let Some((id, sub_score)) = find_best_match(sub, input) {
            return Some((id, sub_score));
        }
    }

    None
}




/// ユーザーの発言から近いタスクタイトルを見つけて、そのIDを返す
pub fn find_task_id_by_title_fuzzy(input: &str) -> Option<u32> {
    let tasks = load_tasks();

    // 全部小文字にして一致確認
    let input_lower = input.to_lowercase();

    for task in &tasks {
        if task.title.to_lowercase().contains(&input_lower) && !task.done {
            return Some(task.id);
        }
        // サブタスクも再帰的に探す
        if let Some(id) = find_in_subtasks(&task.subtasks, &input_lower) {
            return Some(id);
        }
    }

    None
}

fn find_in_subtasks(subtasks: &[Task], input: &str) -> Option<u32> {
    for task in subtasks {
        if task.title.to_lowercase().contains(input) && !task.done {
            return Some(task.id);
        }
        if let Some(id) = find_in_subtasks(&task.subtasks, input) {
            return Some(id);
        }
    }
    None
}


fn mark_task_done(tasks: &mut [Task], task_id: u32) -> bool {
    for task in tasks {
        if task.id == task_id {
            task.done = true;
            task.status = TaskStatus::Completed;
            return true;
        }
        if mark_task_done(&mut task.subtasks, task_id) {
            return true;
        }
    }
    false
}



pub async fn mark_done(task_id: u32) {
    let mut tasks = load_tasks();
    if mark_task_done(&mut tasks, task_id) {
        save_tasks(&tasks);
        println!("✅ タスク {} を完了にしました。", task_id);
        let response = format!("タスク {} を完了にしました。", task_id);
        let _ = tts::speak(&response).await;
    } else {
        println!("⚠️ タスク {} が見つかりませんでした。", task_id);
        let response = format!("タスク {} は見つかりませんでした。", task_id);
        let _ = tts::speak(&response).await;
    }
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
            due_date: None,
            priority: None,
            status: TaskStatus::NotStarted,
            visibility: Visibility::Visible,
            notes: None,
            tags: vec![],
            subtasks: vec![],
            extensions: Map::new(),
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
        due_date: None,
        priority: None,
        status: TaskStatus::NotStarted,
        visibility: Visibility::Visible,
        notes: None,
        tags: vec![],
        subtasks: vec![],
        extensions: Map::new(),
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

    tasks.push(Task { 
        id: 1, 
        title: "一件目".to_string(), 
        done: false,
        due_date: None,
        priority: None,
        status: TaskStatus::NotStarted,
        visibility: Visibility::Visible,
        notes: None,
        tags: vec![],
        subtasks: vec![],
        extensions: Map::new(),
    });

    tasks.push(Task { 
        id: 2, 
        title: "二件目".to_string(), 
        done: false,
        due_date: None,
        priority: None,
        status: TaskStatus::NotStarted,
        visibility: Visibility::Visible,
        notes: None,
        tags: vec![],
        subtasks: vec![],
        extensions: Map::new(),
    });

    save_tasks_with_file(file, &tasks);
    let loaded = load_tasks_with_file(file);

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].title, "一件目");
    assert_eq!(loaded[1].title, "二件目");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Task, TaskStatus, Visibility};
    use chrono::NaiveDate;

    fn dummy_task(id: u32, title: &str) -> Task {
        Task {
            id,
            title: title.to_string(),
            done: false,
            due_date: None,
            priority: None,
            status: TaskStatus::Pending,
            visibility: Visibility::Normal,
            notes: None,
            tags: vec![],
            subtasks: vec![],
            extensions: Map::new(),
        }
    }

    #[test]
    fn test_similarity_finds_exact_task() {

        let tasks = vec![
            dummy_task(1, "週報提出"),
            dummy_task(2, "資料作成"),
        ];

        // 上書き保存テスト用
        save_tasks_with_file("test_similarity.json", &tasks);
        set_task_file("test_similarity.json");

        let found = find_task_id_by_similarity("週報出したよ", 0.75);
        assert_eq!(found, Some(1));

        std::fs::remove_file("test_similarity.json").ok();
    }
    #[tokio::test]
    async fn test_similarity_logs_best_score() {
        // 1. 先にファイルパスをセット
        set_task_file("test_best_score.json");
    
        // 2. 仮タスクを作成
        let tasks = vec![
            Task {
                id: 1,
                title: "週報提出".to_string(),
                done: false,
                due_date: None,
                priority: None,
                status: TaskStatus::Pending,
                visibility: Visibility::Normal,
                notes: None,
                tags: vec![],
                subtasks: vec![],
                extensions: Map::new(),
            },
            Task {
                id: 2,
                title: "資料作成".to_string(),
                done: false,
                due_date: None,
                priority: None,
                status: TaskStatus::Pending,
                visibility: Visibility::Normal,
                notes: None,
                tags: vec![],
                subtasks: vec![],
                extensions: Map::new(),
            },
        ];
    
        // 3. そのファイルに保存
        save_tasks_with_file(get_task_file(), &tasks);
    
        // 4. 類似度テスト
        let found = find_task_id_by_similarity("週報出したよ", 0.7);
        assert_eq!(found, Some(1));
    
        // 5. クリーンアップ
        std::fs::remove_file("test_best_score.json").ok();
    }
}    
}
