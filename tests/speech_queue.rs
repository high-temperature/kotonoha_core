use std::time::Duration;

use kotonoha_core::speech::{SpeechKind, SpeechQueue};
use kotonoha_core::tts;

use std::sync::{Mutex, OnceLock};

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}


#[tokio::test]
async fn user_has_priority_over_monologue() {
    let _g = test_lock();        // ←追加：このスコープ中は他テストが入れない
    let _ = tts::take_spoken();  // ←追加：前のテストの残りをクリア

    // 実音を鳴らさない
    tts::enable_mock_mode();

    // テストを単純化（抑制なし、クールダウンなし）
    let speech = SpeechQueue::spawn(Duration::from_secs(0), Duration::from_secs(0));

    // 先に独り言 → 後にユーザー応答
    speech.say(SpeechKind::Monologue, "mono").await;
    speech.say(SpeechKind::User, "user").await;

    // 非同期ワーカーが処理する時間を少し与える
    tokio::time::sleep(Duration::from_millis(50)).await;

    let spoken = tts::take_spoken();
    assert!(!spoken.is_empty(), "nothing was spoken");
    assert_eq!(spoken[0], "user");
}

#[tokio::test]
async fn alert_has_priority_over_monologue() {
    let _g = test_lock();        // ←追加：このスコープ中は他テストが入れない
    let _ = tts::take_spoken();  // ←追加：前のテストの残りをクリア

    tts::enable_mock_mode();

    let speech = SpeechQueue::spawn(Duration::from_secs(0), Duration::from_secs(0));

    speech.say(SpeechKind::Monologue, "mono").await;
    speech.say(SpeechKind::Alert, "alert").await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let spoken = tts::take_spoken();
    assert!(!spoken.is_empty(), "nothing was spoken");
    assert_eq!(spoken[0], "alert");
}
