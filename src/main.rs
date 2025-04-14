use dotenvy::dotenv;
use std::env;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	let api_key = env::var("OPENAI_API_KEY")?;

	let client = reqwest::Client::new();
	let mut messages = vec![];

	println!("kotonoha とお話ししましょう。終了したいときは'exit'と入力してください。");

	loop{
		print!("あなた >");
		io::stdout().flush()?;
		let mut user_input = String::new();
		io::stdin().read_line(&mut user_input)?;
		let user_input = user_input.trim();
		if user_input == "exit"{
			break;
		}

		messages.push(ChatMessage{
			role: "user".into(),
			content: user_input.to_string(),
		});

		let request_body = ChatRequest {
            model: "gpt-3.5-turbo".into(),
            messages: messages.clone(),
        };

		let response = client
			.post("https://api.openai.com/v1/chat/completions")
			.bearer_auth(&api_key)
			.json(&request_body)
			.send()
			.await?;

		let text = response.text().await?;
		println!("📦 ChatGPTの応答内容: {}", text);

		let resp_json: Result<ChatResponse, serde_json::Error> = serde_json::from_str(&text);
		if let Ok(resp) = resp_json{
			let reply = &resp.choices[0].message.content;
			println!("Kototnoha > {}",reply);
			messages.push(ChatMessage{
				role: "assistant".into(),
				content: reply.clone(),
			});
		} else{
			println!("ChatGPT API からの返答に失敗しました。\nメッセージ内容{}", text);
			break;
		}

	}

	Ok(())
}



#[derive(Serialize, Clone)]
struct ChatRequest{
	model: String,
	messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage{
	role: String,
	content: String,
}

#[derive(Deserialize)]
struct ChatResponse{
	choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice{
	message: ChatMessage,
}