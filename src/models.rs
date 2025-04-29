use serde::{Serialize, Deserialize};
use chrono::NaiveDate;
use serde_json::Value;
use serde_json::Map;

#[derive(Serialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskStatus {
    NotStarted,    // 未着手
    InProgress,    // 進行中
    Pending,    // 保留中
    OnHold,        // 保留
    Completed,     // 完了
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Visibility {
    Visible,       // 表示中
    Normal,        // 非表示
    Hidden,        // 非表示
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u32,                            // タスクID
    pub title: String,                      // タイトル
    pub done: bool,                         // 完了済みか
    pub due_date: Option<NaiveDate>,         // 締切日（なければNone）
    pub priority: Option<u8>,                // 優先度（1:最高 〜 5:低いなど）
    pub status: TaskStatus,                  // 状態（未着手・進行中など）
    pub visibility: Visibility,              // 表示/非表示
    pub notes: Option<String>,               // 詳細メモ
    pub tags: Vec<String>,                   // タグ
    pub subtasks: Vec<Task>,                 // サブタスク（入れ子構造）
    pub extensions: Map<String, Value>,      // プラグイン拡張領域
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_response_deserialization() {
        let raw = r#"
        {
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "タスクを追加しました。"
                }
            }]
        }
        "#;

        let parsed: ChatResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.choices[0].message.content, "タスクを追加しました。");
    }
}