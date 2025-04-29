use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;

use std::time::Duration;
use std::thread::sleep;

#[test]
fn test_cli_add_and_complete_task() {
    let task_file = "tasks_test_cli.json";
    let _ = fs::remove_file(task_file); // 前回の残骸を削除

    let mut child = Command::new("target/debug/kotonoha_core")
        .env("TASK_FILE", task_file)
        .env("MOCK_TTS", "1") 
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start kotonoha_core");
    sleep(Duration::from_millis(1000));

    if let Some(stdin) = child.stdin.as_mut() {
        writeln!(stdin, "統合テストタスクをするの覚えておいて").expect("failed to write to stdin");
        writeln!(stdin, "統合テストタスクが完了しました。").expect("failed to write to stdin");
        writeln!(stdin, "exit").expect("failed to write to stdin");
    } else {
        panic!("stdin not available");
    }
    let output = child.wait_with_output().expect("failed to read output");
    println!("status: {:?}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "process exited with failure");

    // ファイルに保存されているか確認
    let data = fs::read_to_string(task_file).expect("failed to read task file");
    assert!(data.contains("統合テストタスク"));
    assert!(data.contains("\"done\": true"));

    let _ = fs::remove_file(task_file); // クリーンアップ
}
