use serde::{Deserialize, Serialize};
use std::{sync::Mutex, time::Duration};
#[derive(Default)]
pub struct ChatState(pub Mutex<String>);
#[tauri::command]
pub fn configure_chat(key: String, state: tauri::State<ChatState>) -> Result<(), String> {
    if key.len() > 512 || key.trim().is_empty() {
        return Err("请填写有效 API Key".into());
    }
    *state.0.lock().map_err(|_| "密钥状态不可用")? = key.trim().to_string();
    Ok(())
}
#[derive(Deserialize, Serialize)]
pub struct Message {
    role: String,
    content: String,
}
#[tauri::command]
pub async fn chat_reply(
    messages: Vec<Message>,
    state: tauri::State<'_, ChatState>,
) -> Result<String, String> {
    if messages.is_empty()
        || messages.len() > 21
        || messages.iter().any(|m| {
            !matches!(m.role.as_str(), "user" | "assistant")
                || m.content.trim().is_empty()
                || m.content.chars().count() > 2000
        })
    {
        return Err("聊天参数无效，最多 21 条，每条 2000 字".into());
    }
    let key = state.0.lock().map_err(|_| "密钥状态不可用")?.clone();
    if key.is_empty() {
        return Err("请先设置聊天 API Key".into());
    }
    let mut input = vec![
        serde_json::json!({"role":"system","content":"你是桌面助手澜鲸，温柔自然，用中文回应，最多100字。不声称执行未执行的电脑操作。"}),
    ];
    input.extend(messages.into_iter().map(|m| serde_json::json!(m)));
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|_| "初始化失败")?;
    let mut response=client.post("https://api.deepseek.com/chat/completions").bearer_auth(key).json(&serde_json::json!({"model":"deepseek-flash","messages":input,"thinking":{"type":"disabled"},"max_tokens":512,"stream":false})).send().await.map_err(|_|"聊天请求失败或超时，请检查网络")?;
    if !response.status().is_success() {
        return Err(format!(
            "聊天服务 HTTP {}，请检查密钥、余额或稍后重试",
            response.status().as_u16()
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "聊天响应读取失败")? {
        if bytes.len() + chunk.len() > 262144 {
            return Err("聊天响应过大".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let data: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "聊天响应不是 JSON")?;
    if data["choices"][0]["finish_reason"] != "stop" {
        return Err("回复未完整生成，请重试".into());
    }
    let text = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("回复为空")?
        .trim();
    if text.is_empty() {
        return Err("回复为空".into());
    }
    Ok(text.chars().take(300).collect())
}
#[tauri::command]
pub async fn gpu_speech(address: String, text: String) -> Result<Vec<u8>, String> {
    if text.trim().is_empty() || text.chars().count() > 300 {
        return Err("GPU 朗读限 1–300 字".into());
    }
    let url = crate::gateway::local_endpoint(&address)?
        .join("speech")
        .map_err(|_| "地址无效")?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|_| "初始化失败")?;
    let mut response = client
        .post(url)
        .json(&serde_json::json!({"text":text}))
        .send()
        .await
        .map_err(|_| "GPU 服务不可达或合成超时，未自动重试")?;
    if !response.status().is_success() {
        return Err(format!("GPU 服务 HTTP {}", response.status().as_u16()));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "音频读取失败")? {
        if bytes.len() + chunk.len() > 8388608 {
            return Err("音频超过 8 MiB".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("服务没有返回 WAV 音频".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn gpu_rejects_remote_and_empty() {
        assert!(gpu_speech("http://example.com".into(), "你好".into())
            .await
            .is_err());
        assert!(gpu_speech("http://127.0.0.1:9881".into(), "".into())
            .await
            .is_err());
    }
    #[tokio::test]
    async fn gpu_accepts_wav() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut data = [0; 4096];
            let count = socket.read(&mut data).unwrap();
            assert!(String::from_utf8_lossy(&data[..count]).starts_with("POST /speech "));
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\nConnection: close\r\n\r\nRIFF1234WAVE").unwrap();
        });
        assert_eq!(
            gpu_speech(address, "你好".into()).await.unwrap(),
            b"RIFF1234WAVE"
        );
        handle.join().unwrap();
    }
}
