use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_cli_add_and_complete_task() {
    // ユニークなファイル名で競合防止
    let task_file = format!("tasks_test_cli_{}.json", Uuid::new_v4());
    let _ = fs::remove_file(&task_file); // 前回の残骸を削除

    let mut child = Command::new("target/debug/kotonoha_core")
        .env("TASK_FILE", &task_file)
        .env("MOCK_TTS", "1")
        .env("MOCK_OPENAI", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start kotonoha_core");

    // 少し待機してプロセスが立ち上がるのを待つ
    sleep(Duration::from_millis(500));

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");

        writeln!(stdin, "統合テストタスクをするの覚えておいて").unwrap();
        stdin.flush().unwrap();
        sleep(Duration::from_millis(300));

        writeln!(stdin, "統合テストタスクが完了しました。").unwrap();
        stdin.flush().unwrap();
        sleep(Duration::from_millis(300));

        writeln!(stdin, "exit").unwrap();
        stdin.flush().unwrap();
    }

    let output = child.wait_with_output().expect("failed to read output");

    // デバッグ出力
    println!("status: {:?}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // ファイル出力の確認（リトライ付き）
    let mut retry = 0;
    let max_retry = 5;
    let task_data = loop {
        match fs::read_to_string(&task_file) {
            Ok(data) => break data,
            Err(_) if retry < max_retry => {
                retry += 1;
                sleep(Duration::from_millis(200));
            }
            Err(e) => panic!("failed to read task file: {}", e),
        }
    };

    // 内容確認
    assert!(task_data.contains("統合テストタスク"));
    assert!(task_data.contains("\"done\": true"));

    let _ = fs::remove_file(&task_file); // 後始末
}
