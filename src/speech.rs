use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeechKind {
    User,       // ユーザーへの応答（最優先）
    Alert,      // 期限通知など（次点）
    Monologue,  // 独り言（最下位）
}

#[derive(Debug, Clone)]
pub struct SpeechRequest {
    pub kind: SpeechKind,
    pub text: String,
}

#[derive(Clone)]
pub struct SpeechQueue {
    tx: mpsc::Sender<SpeechRequest>,
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    last_user_action: Option<Instant>,
    last_monologue_spoken: Option<Instant>,
}

impl SpeechQueue {
    /// 発話ワーカーを起動して、送信用ハンドルを返す
    pub fn spawn(
        monologue_cooldown: Duration,
        suppress_monologue_after_user: Duration,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<SpeechRequest>(64);

        let state = Arc::new(Mutex::new(State::default()));
        let state_worker = state.clone();

        // 優先度別キュー
        let q_user: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
        let q_alert: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
        let q_mono: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));

        let q_user_w = q_user.clone();
        let q_alert_w = q_alert.clone();
        let q_mono_w = q_mono.clone();

        tokio::spawn(async move {
            loop {
                // まず受信をなるべく捌く（バースト耐性）
                while let Ok(req) = rx.try_recv() {
                    match req.kind {
                        SpeechKind::User => {
                            // ユーザーが喋った/入力した扱い（独り言抑制に使う）
                            {
                                let mut st = state_worker.lock().await;
                                st.last_user_action = Some(Instant::now());
                            }
                            q_user_w.lock().await.push_back(req.text);
                        }
                        SpeechKind::Alert => {
                            q_alert_w.lock().await.push_back(req.text);
                        }
                        SpeechKind::Monologue => {
                            q_mono_w.lock().await.push_back(req.text);
                        }
                    }
                }

                // 優先度順で1件取り出す
                let next = {
                    if let Some(s) = q_user_w.lock().await.pop_front() {
                        Some((SpeechKind::User, s))
                    } else if let Some(s) = q_alert_w.lock().await.pop_front() {
                        Some((SpeechKind::Alert, s))
                    } else if let Some(s) = q_mono_w.lock().await.pop_front() {
                        Some((SpeechKind::Monologue, s))
                    } else {
                        None
                    }
                };

                if let Some((kind, text)) = next {
                    // 独り言は「ユーザー操作直後」や「クールダウン中」なら黙る（捨てる）
                    if kind == SpeechKind::Monologue {
                        let now = Instant::now();
                        let st = state_worker.lock().await;

                        if let Some(t) = st.last_user_action {
                            if now.duration_since(t) < suppress_monologue_after_user {
                                // 邪魔なので捨てる
                                continue;
                            }
                        }
                        if let Some(t) = st.last_monologue_spoken {
                            if now.duration_since(t) < monologue_cooldown {
                                // 喋りすぎなので捨てる
                                continue;
                            }
                        }
                        drop(st);

                        // ここで「喋った」記録
                        let mut st = state_worker.lock().await;
                        st.last_monologue_spoken = Some(now);
                    }

                    // 実際の発話（MOCK_TTS含め tts::speak に集約）
                    // エラーは落とさずログだけ
                    if let Err(e) = crate::tts::speak(&text).await {
                        eprintln!("TTS failed: {}", e);
                    }

                    continue;
                }

                // 何もなければ「次の受信」を待つ
                match rx.recv().await {
                    Some(req) => {
                        // 1件だけ入れて次ループでまとめて処理
                        match req.kind {
                            SpeechKind::User => {
                                let mut st = state_worker.lock().await;
                                st.last_user_action = Some(Instant::now());
                                drop(st);
                                q_user_w.lock().await.push_back(req.text);
                            }
                            SpeechKind::Alert => q_alert_w.lock().await.push_back(req.text),
                            SpeechKind::Monologue => q_mono_w.lock().await.push_back(req.text),
                        }
                    }
                    None => break, // sender全破棄で終了
                }
            }
        });

        Self { tx, state }
    }

    pub async fn say(&self, kind: SpeechKind, text: impl Into<String>) {
        // 送信失敗はワーカー停止なので握りつぶし
        let _ = self.tx.send(SpeechRequest { kind, text: text.into() }).await;
    }

    pub async fn say_user(&self, text: impl Into<String>) { self.say(SpeechKind::User, text).await }
    pub async fn say_alert(&self, text: impl Into<String>) { self.say(SpeechKind::Alert, text).await }
    pub async fn say_monologue(&self, text: impl Into<String>) { self.say(SpeechKind::Monologue, text).await }

    /// ユーザー操作があったことだけ記録したい場合に使う（将来：GUIのクリック等）
    pub async fn mark_user_action(&self) {
        let mut st = self.state.lock().await;
        st.last_user_action = Some(Instant::now());
    }
}
