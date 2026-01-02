use crate::models::{Task, TaskStatus, Visibility};

use crate::tts;

use std::fs::{File, OpenOptions};
use std::io::{BufReader,BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use once_cell::sync::OnceCell;

use serde_json::Map;

use strsim::jaro_winkler;

static TASK_FILE: OnceCell<Mutex<Option<String>>> = OnceCell::new();

pub const DEFAULT_TASK_FILE: &str = "tasks.json";

pub fn set_task_file(file: &str) {
    let lock = TASK_FILE.get_or_init(|| Mutex::new(None));
    *lock.lock().unwrap() = Some(file.to_string());
}

fn get_task_file() -> String {
    TASK_FILE
        .get()
        .and_then(|lock| lock.lock().unwrap().clone())
        .unwrap_or_else(|| DEFAULT_TASK_FILE.to_string())
}

pub fn load_tasks<P: AsRef<Path>>(path: Option<P>) -> Vec<Task> {
    let path = match path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(get_task_file()),
    };

    load_tasks_with_file(&path)
}

pub fn load_tasks_with_file(path: &Path) -> Vec<Task> {
    if !path.exists() {
        return vec![];
    }

    let file = File::open(path).expect("Failed to open tasks file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_else(|e| {
    eprintln!("Failed to parse tasks file: {} ({})", path.display(), e);
    vec![]
})

}

pub fn save_tasks<P: AsRef<Path>>(path: Option<P>, tasks: &[Task]) {
    let path = match path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(get_task_file()),
    };

    save_tasks_with_file(&path, tasks);
}

pub fn save_tasks_with_file(path: &Path, tasks: &[Task]) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .expect("Failed to open tasks file for writing");

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, tasks).expect("Failed to write tasks to file");
}


pub async fn add_task(title: &str) {
    let mut tasks = load_tasks::<&str>(None);
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
    save_tasks::<&str>(None, &tasks);

    println!("Kotonoha > タスク「{}」を登録しました。", title);
    let response = format!("タスクを「{}」を登録しました。", title);
    let _ = tts::speak(&response).await;
}
pub async fn list_tasks() {
    list_tasks_in::<&str>(None).await;
}  
pub async fn list_tasks_in<P:AsRef<Path>>(path: Option<P>) {
    let tasks = load_tasks(path);

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

pub async fn mark_done(task_id: u32)
{
    mark_done_in::<&str>(None, task_id).await;
}


pub async fn mark_done_in<P:AsRef<Path>>(path: Option<P>, task_id: u32) {
    let mut tasks = load_tasks(path);
    if mark_task_done(&mut tasks, task_id) {
        save_tasks::<&str>(None, &tasks);
        println!("✅ タスク {} を完了にしました。", task_id);
        let response = format!("タスク {} を完了にしました。", task_id);
        let _ = tts::speak(&response).await;
    } else {
        println!("⚠️ タスク {} が見つかりませんでした。", task_id);
        let response = format!("タスク {} は見つかりませんでした。", task_id);
        let _ = tts::speak(&response).await;
    }
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



pub fn find_task_id_by_similarity_from_tasks(tasks: &[Task], input: &str, threshold: f64) -> Option<u32> {
    let mut best_match = None;
    let mut best_score = 0.0; // 初期スコアを0.0にする

    println!("🔍 入力: \"{}\"", input);

    for task in tasks {
        if let Some((id, title ,score)) = find_best_match(task, input) {
            println!("📝 id\"{}\" タスク \"{}\" のスコア: {:.3}", id, title, score);

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

/// 任意のファイルパスでタスクを読み込んで、類似度検索する
pub fn find_task_id_by_similarity_in<P: AsRef<Path>>(path: Option<P>, input: &str, threshold: f64) -> Option<u32> {
    let tasks = match path {
        Some(ref p) => load_tasks(Some(p.as_ref())),
        None => load_tasks::<&str>(None),
    };
    find_task_id_by_similarity_from_tasks(&tasks, input, threshold)
}


/// デフォルトファイル（tasks.json）で類似度検索する
pub fn find_task_id_by_similarity(input: &str, threshold: f64) -> Option<u32> {
    find_task_id_by_similarity_in::<&str>(None, input, threshold)
}



/// ユーザーの発言から近いタスクタイトルを見つけて、そのIDを返す
pub fn find_task_id_by_title_fuzzy(input: &str) -> Option<u32> {
    let tasks = load_tasks::<&str>(None);

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

fn find_best_match(task: &Task, input: &str) -> Option<(u32, String, f64)> {
    let score = jaro_winkler(&task.title.to_lowercase(), &input.to_lowercase());

    if !task.done {
        return Some((task.id, task.title.clone(), score)); // ← ★ スコアも返す！
    }

    for sub in &task.subtasks {
        if let Some((id, title, sub_score)) = find_best_match(sub, input) {
            return Some((id, title, sub_score)); // ← ★ こっちも3つ返す！
        }
    }

    None
}

pub fn find_task_with_score(input: &str, threshold: f64) -> Option<(u32, String, f64)> {
    let tasks = load_tasks::<&str>(None);
    let mut best_match: Option<(u32, String, f64)> = None;
    let mut best_score = 0.0;

    for task in &tasks {
        if let Some((id, title, score)) = find_best_match(task, input) {
            if score > best_score {
                best_match = Some((id, title, score));
                best_score = score;
            }
        }
    }

    if let Some((id, title, score)) = &best_match {
        if *score >= threshold {
            return Some((id.clone(), title.clone(), *score));
        }
    }

    None
}





/// タスク一覧をまとめた文字列を返す
pub fn summarize_tasks_for_prompt() -> String {
    let tasks = load_tasks::<&str>(None);
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

use chrono::{Local, NaiveDate};
/// 期限が within_days 日以内の未完了タスクを返す
pub fn find_due_within_days(within_days: i64) -> Vec<Task> {
    let today: NaiveDate = Local::now().date_naive();
    let limit = today + chrono::Duration::days(within_days);

    let tasks = load_tasks::<&str>(None);
    let mut out = Vec::new();

    fn walk(out: &mut Vec<Task>, tasks: &[Task], today: NaiveDate, limit: NaiveDate) {
        for t in tasks {
            if !t.done {
                if let Some(due) = t.due_date {
                    if due >= today && due <= limit {
                        out.push(t.clone());
                    }
                }
            }
            walk(out, &t.subtasks, today, limit);
        }
    }

    walk(&mut out, &tasks, today, limit);
    out
}


#[cfg(test)]
mod tests {
    // tests/tasks_tests.rs

    use crate::tasks::*;
    use crate::models::{Task, TaskStatus, Visibility};
    use uuid::Uuid;
    use std::fs;
    use std::path::PathBuf;

    /// テスト専用の一時タスクファイル
    pub struct TempTaskFile {
        path: PathBuf,
    }

    impl TempTaskFile {
        pub fn new() -> Self {
            let filename = format!("tasks_test_{}.json", Uuid::new_v4());
            let path = PathBuf::from(filename);
            set_task_file(path.to_str().unwrap());
            Self { path }
        }

        pub fn path(&self) -> &str {
            self.path.to_str().unwrap()
        }

        pub fn save(&self, tasks: &[Task]) {
            save_tasks(Some(&self.path), tasks);
        }
    
        pub fn load(&self) -> Vec<Task> {
            load_tasks(Some(&self.path))
        }    
    }

    
    impl Drop for TempTaskFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

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
            extensions: serde_json::Map::new(),
        }
    }

    #[test]
    fn test_add_and_load_tasks() {
        let temp = TempTaskFile::new();

        
        let mut tasks = temp.load();
        tasks.push(dummy_task(1, "テストタスク"));
        temp.save(&tasks);

        let loaded = temp.load();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "テストタスク");
    }

    #[test]
    fn test_mark_done_updates_task() {
        let temp = TempTaskFile::new();

        let tasks = vec![dummy_task(1, "完了チェック")];
        temp.save(&tasks);

        let mut loaded = temp.load();
        if let Some(task) = loaded.iter_mut().find(|t| t.id == 1) {
            task.done = true;
        }
        temp.save(&loaded);

        let updated = temp.load();
        let updated_task = updated.iter().find(|t| t.id == 1).expect("タスクが見つかりません");
        assert!(updated_task.done);
    }

    #[test]
    fn test_add_multiple_tasks_and_order() {
        let temp = TempTaskFile::new();

        let tasks = vec![
            dummy_task(1, "一件目"),
            dummy_task(2, "二件目"),
        ];
        temp.save(&tasks);

        let loaded = temp.load();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].title, "一件目");
        assert_eq!(loaded[1].title, "二件目");
    }

    mod similarity_tests {
        use super::*;

        #[tokio::test]
        async fn test_similarity_logs_best_score() {
            let temp = TempTaskFile::new();

            let tasks = vec![
                dummy_task(1, "週報提出"),
                dummy_task(2, "資料作成"),
            ];
            temp.save(&tasks);

            let found = find_task_id_by_similarity_in(Some(temp.path()), "週報出したよ", 0.7);            assert_eq!(found, Some(1));
        }
    }

}
