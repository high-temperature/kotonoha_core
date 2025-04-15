// src/tts.rs

use reqwest::Client;
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

const KASUKABE_TSUMUGI_ID: &str = "8"; // 春日部つむぎのID

pub async fn speak(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // クエリを作成（音声合成準備）
    let query = client
        .post("http://127.0.0.1:50021/audio_query")
        .query(&[("text", text), ("speaker", KASUKABE_TSUMUGI_ID)]) // speaker 8 = 春日部つむぎ
        .send()
        .await?
        .text()
        .await?;

    // 音声合成（合成されたWAV）
    let audio = client
        .post("http://127.0.0.1:50021/synthesis")
        .query(&[("speaker", KASUKABE_TSUMUGI_ID)]) // speaker 8 = 春日部つむぎ
        .header("Content-Type", "application/json")
        .body(query)
        .send()
        .await?
        .bytes()
        .await?;

    // 再生
    let (_stream, handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&handle)?;
    let source = Decoder::new(Cursor::new(audio))?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
