use std::process::{Command, Stdio};
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
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start kotonoha_core");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    let mut stdout = child.stdout.take().expect("failed to open stdout");

    use std::sync::{Arc, Mutex};

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let buffer_clone = Arc::clone(&buffer);

    let reader = std::thread::spawn(move || {
        use std::io::Read;
        let mut stdout = stdout;
        let mut local_buf = Vec::new();
        stdout.read_to_end(&mut local_buf).expect("failed to read stdout");

        let mut shared = buffer_clone.lock().unwrap();
        *shared = local_buf;
    });


    use std::io::Write;

    fn wait_for_prompt(shared: &Arc<Mutex<Vec<u8>>>) -> bool {
        let binding = shared.lock().unwrap();
        let text = String::from_utf8_lossy(&binding);
        text.contains("あなた >")
    }
    

    // 簡単なポーリング
    std::thread::sleep(std::time::Duration::from_millis(300));

    // まず最初のプロンプトを待つ
    loop {
        if wait_for_prompt(&buffer) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    

    writeln!(stdin, "統合テストタスクをするの覚えておいて").expect("failed to write to stdin");

    std::thread::sleep(std::time::Duration::from_millis(500));

    writeln!(stdin, "統合テストタスクが完了しました。").expect("failed to write to stdin");

    std::thread::sleep(std::time::Duration::from_millis(500));

    writeln!(stdin, "exit").expect("failed to write to stdin");

    let output = child.wait_with_output().expect("failed to wait on child");

    println!("status: {:?}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "Process exited with failure");

    let data = fs::read_to_string(task_file).expect("failed to read task file");
    assert!(data.contains("統合テストタスク"));
    assert!(data.contains("\"done\": true"));

    let _ = fs::remove_file(task_file); // クリーンアップ
}
