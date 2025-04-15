use kotonoha_core::tasks;
use kotonoha_core::models::Task;

const TEST_FILE: &str = "tasks_test_integration.json";

#[test]
fn test_add_and_mark_task_flow() {
    let _ = std::fs::remove_file(TEST_FILE);

    // 1. タスクを追加
    let task_list = vec![Task {
        id: 1,
        title: "統合テストのタスク".to_string(),
        done: false,
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
