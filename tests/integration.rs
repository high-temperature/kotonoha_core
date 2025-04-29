use kotonoha_core::tasks;
use kotonoha_core::models::Task;
use kotonoha_core::models::TaskStatus;
use kotonoha_core::models::Visibility;
use serde_json::Map;

const TEST_FILE: &str = "tasks_test_integration.json";

#[test]
fn test_add_and_mark_task_flow() {
    let _ = std::fs::remove_file(TEST_FILE);

    // 1. タスクを追加
    let task_list = vec![
        Task {
            id: 1,
            title: "統合テストタスク".to_string(),
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
    tasks::save_tasks_with_file(TEST_FILE, &task_list);

    // 2. 読み込んでチェック
    let loaded = tasks::load_tasks_with_file(TEST_FILE);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].done, false);

    // 3. 完了にして保存
    let mut updated = loaded;
    updated[0].done = true;
    tasks::save_tasks_with_file(TEST_FILE, &updated);

    // 4. 再ロードして完了確認
    let confirmed = tasks::load_tasks_with_file(TEST_FILE);
    assert!(confirmed[0].done);

    std::fs::remove_file(TEST_FILE).ok();

}


#[test]
fn test_multiple_tasks_add_and_check() {
    let test_file = "tasks_test_multi.json";
    let _ = std::fs::remove_file(test_file);

    let tasks = vec![
        Task {
            id: 1,
            title: "1つ目のテストタスク".to_string(),
            done: false,
            due_date: None,
            priority: None,
            status: TaskStatus::NotStarted,
            visibility: Visibility::Visible,
            notes: None,
            tags: vec![],
            subtasks: vec![],
            extensions: Map::new(),
        },
        Task {
            id: 2,
            title: "2つ目のテストタスク".to_string(),
            done: false,
            due_date: None,
            priority: None,
            status: TaskStatus::NotStarted,
            visibility: Visibility::Visible,
            notes: None,
            tags: vec![],
            subtasks: vec![],
            extensions: Map::new(),
        },
        Task {
            id: 3,
            title: "3つ目のテストタスク".to_string(),
            done: true,
            due_date: None,
            priority: None,
            status: TaskStatus::NotStarted,
            visibility: Visibility::Visible,
            notes: None,
            tags: vec![],
            subtasks: vec![],
            extensions: Map::new(),
        },
    ];
    tasks::save_tasks_with_file(test_file, &tasks);

    let loaded = tasks::load_tasks_with_file(test_file);
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded[0].title, "1つ目のテストタスク");
    assert!(loaded[2].done);

    // 未完了タスクだけを確認
    let pending: Vec<&Task> = loaded.iter().filter(|t| !t.done).collect();
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].title, "1つ目のテストタスク");
    assert_eq!(pending[1].title, "2つ目のテストタスク");

    std::fs::remove_file(test_file).ok();
}
