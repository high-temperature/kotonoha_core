// src/tts.rs


#[cfg(feature = "tts")]
use rodio::{Decoder, OutputStream, Sink};

#[cfg(feature = "tts")]
use std::io::Cursor;

#[cfg(feature = "tts")]
use reqwest::Client;

#[cfg(feature = "tts")]
const KASUKABE_TSUMUGI_ID: &str = "8"; // 春日部つむぎのID


use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::env;

static MOCK_MODE: Lazy<AtomicBool> = Lazy::new(|| {
    AtomicBool::new(env::var("MOCK_TTS").is_ok())
});


// これを main.rs から呼ぶ
pub fn enable_mock_mode() {
    MOCK_MODE.store(true, Ordering::Relaxed);
}

#[cfg(test)]
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
static SPOKEN: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

#[cfg(test)]
pub fn take_spoken() -> Vec<String> {
    let m = SPOKEN.get_or_init(|| Mutex::new(Vec::new()));
    let mut v = m.lock().unwrap();
    std::mem::take(&mut *v)
}


#[cfg(not(feature = "tts"))]
pub async fn speak(_text: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(feature = "tts")]
pub async fn speak(text: &str) -> Result<(), Box<dyn std::error::Error>> {
if MOCK_MODE.load(Ordering::Relaxed) {
    // モックモードなら、VOICEVOXには繋がずプリントする
    println!("[MOCK VOICE]: {}", text);

    // ★テスト時だけ「喋った内容」を記録する
    #[cfg(test)]
    {
        let m = SPOKEN.get_or_init(|| Mutex::new(Vec::new()));
        m.lock().unwrap().push(text.to_string());
    }

    return Ok(());
}


    // 本物のVoiceVoxを呼ぶ処理
    let client = Client::new();

    let query = client
        .post("http://127.0.0.1:50021/audio_query")
        .query(&[("text", text), ("speaker", KASUKABE_TSUMUGI_ID)])
        .send()
        .await?
        .text()
        .await?;

    let audio = client
        .post("http://127.0.0.1:50021/synthesis")
        .query(&[("speaker", KASUKABE_TSUMUGI_ID)])
        .header("Content-Type", "application/json")
        .body(query)
        .send()
        .await?
        .bytes()
        .await?;

    let (_stream, handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&handle)?;
    let source = Decoder::new(Cursor::new(audio))?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
