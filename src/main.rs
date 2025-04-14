use reqwest::Client;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;
use rodio::{Decoder, OutputStream, Sink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let text = "この番組は、御覧のスポンサーの提供で、お送りします";

    // 1. クエリを作成（text→音声変換の準備）
    let query_resp = client.post("http://127.0.0.1:50021/audio_query")
        .query(&[("text", text), ("speaker", "8")]) // speaker=8 は春日部つむぎ（のはず）
        .send()
        .await?
        .text()
        .await?;

    // 2. 音声合成（JSONクエリ→音声データ）
    let synth_resp = client.post("http://127.0.0.1:50021/synthesis")
        .query(&[("speaker", "1")])
        .header("Content-Type", "application/json")
        .body(query_resp)
        .send()
        .await?
        .bytes()
        .await?;

    // 3. 再生！
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    let source = Decoder::new(Cursor::new(synth_resp))?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
