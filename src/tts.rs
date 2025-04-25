use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::env;
use rodio::{Decoder, OutputStream, Sink};
use reqwest::Client;
use std::io::Cursor;

// 環境変数でモックモードを決める
static MOCK_MODE: Lazy<AtomicBool> = Lazy::new(|| {
    AtomicBool::new(env::var("MOCK_TTS").is_ok())
});

pub async fn speak(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    if MOCK_MODE.load(Ordering::Relaxed) {
        println!("[MOCK TTS]: {}", text);
        return Ok(());
    }

    // 本物のVOICEVOX通信
    let client = Client::new();
    let query_resp = client.post("http://127.0.0.1:50021/audio_query")
        .query(&[("text", text), ("speaker", "8")])
        .send()
        .await?
        .text()
        .await?;

    let synth_resp = client.post("http://127.0.0.1:50021/synthesis")
        .query(&[("speaker", "8")])
        .header("Content-Type", "application/json")
        .body(query_resp)
        .send()
        .await?
        .bytes()
        .await?;

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    let source = Decoder::new(Cursor::new(synth_resp))?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
