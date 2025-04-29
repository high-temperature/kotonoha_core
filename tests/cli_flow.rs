use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;

#[test]
fn test_cli_add_and_complete_task() {
    let task_file = "tasks_test_cli.json";
    let _ = fs::remove_file(task_file); // 前回の残骸を削除

    let mut child = Command::new("target/debug/kotonoha_core")
        .env("TASK_FILE", task_file)
        .env("MOCK_TTS", "1") 
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start kotonoha_core");

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        writeln!(stdin, "統合テストタスクをするの覚えておいて").unwrap();
        writeln!(stdin, "統合テストタスクが完了しました。").unwrap();
        writeln!(stdin, "exit").unwrap(); // 明示的に終了
    }

    let output = child.wait_with_output().expect("failed to read output");
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    println!("STDOUT:\n{}", stdout_str);

    // ファイルに保存されているか確認
    let data = fs::read_to_string(task_file).expect("failed to read task file");
    assert!(data.contains("統合テストタスク"));
    assert!(data.contains("\"done\": true"));

    let _ = fs::remove_file(task_file); // クリーンアップ
}
