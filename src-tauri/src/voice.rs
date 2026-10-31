use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager};
#[derive(Default)]
pub struct VoiceState {
    pub asr: Mutex<Option<Child>>,
    pub speech: Mutex<Option<Child>>,
    pub generation: Arc<AtomicU64>,
}
fn stop(child: &Mutex<Option<Child>>) {
    if let Ok(mut slot) = child.lock() {
        if let Some(mut process) = slot.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}
fn command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}
pub fn shutdown(app: &tauri::AppHandle) {
    let state = app.state::<VoiceState>();
    state.generation.fetch_add(1, Ordering::SeqCst);
    stop(&state.asr);
    stop(&state.speech);
}
#[tauri::command]
pub fn stop_speech(state: tauri::State<VoiceState>) {
    stop(&state.speech);
}
#[tauri::command]
pub fn speak_text(
    text: String,
    rate: i32,
    volume: u32,
    state: tauri::State<VoiceState>,
) -> Result<(), String> {
    if text.trim().is_empty()
        || text.chars().count() > 2000
        || !(-10..=10).contains(&rate)
        || volume > 100
    {
        return Err("文字须为 1–2000 字，语速 -10 至 10，音量 0 至 100".into());
    }
    stop(&state.speech);
    let script = r#"[Console]::InputEncoding=New-Object Text.UTF8Encoding($false);$ErrorActionPreference='Stop';try{$r=[Console]::In.ReadToEnd()|ConvertFrom-Json;Add-Type -AssemblyName System.Speech;$s=New-Object System.Speech.Synthesis.SpeechSynthesizer;$s.Rate=$r.rate;$s.Volume=$r.volume;$s.Speak([string]$r.text);$s.Dispose()}catch{exit 1}"#;
    let mut child = command("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let payload = serde_json::json!({"text":text,"rate":rate,"volume":volume});
    if let Err(e) = child
        .stdin
        .take()
        .ok_or("语音输入管道不可用")?
        .write_all(payload.to_string().as_bytes())
    {
        let _ = child.kill();
        let _ = child.wait();
        return Err(e.to_string());
    }
    *state.speech.lock().map_err(|_| "语音状态不可用")? = Some(child);
    Ok(())
}
#[tauri::command]
pub fn speech_status(state: tauri::State<VoiceState>) -> Result<String, String> {
    let mut slot = state.speech.lock().map_err(|_| "语音状态不可用")?;
    if let Some(child) = slot.as_mut() {
        match child.try_wait().map_err(|e| e.to_string())? {
            None => return Ok("speaking".into()),
            Some(status) => {
                let success = status.success();
                *slot = None;
                return Ok(if success { "completed" } else { "error" }.into());
            }
        }
    }
    Ok("idle".into())
}
#[tauri::command]
pub fn voice_input(
    app: tauri::AppHandle,
    enabled: bool,
    state: tauri::State<VoiceState>,
) -> Result<(), String> {
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    stop(&state.asr);
    if !enabled {
        return Ok(());
    }
    let root = std::path::PathBuf::from(r"E:\codex\live2d形象");
    let python = root.join("runtime/tts-venv/Scripts/python.exe");
    if !python.is_file() || !root.join("runtime/asr/vosk-model-small-cn-0.22").is_dir() {
        return Err("未找到旧项目语音环境或 Vosk 模型".into());
    }
    let script = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("native/voice-chat.py");
    let mut child = command(python.to_str().ok_or("Python 路径无效")?)
        .arg("-u")
        .arg(script)
        .env("ASSISTANT_VOICE_ROOT", root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let output = child.stdout.take().ok_or("识别输出管道不可用")?;
    let guard = state.generation.clone();
    *state.asr.lock().map_err(|_| "识别状态不可用")? = Some(child);
    std::thread::spawn(move || {
        for line in BufReader::new(output).lines().map_while(Result::ok) {
            if guard.load(Ordering::SeqCst) != generation {
                return;
            }
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                if value["event"] == "recording" {
                    stop(&app.state::<VoiceState>().speech);
                }
                let _ = app.emit("voice-input", value);
            }
        }
        if guard.load(Ordering::SeqCst) == generation {
            let _ = app.emit("voice-input", serde_json::json!({"event":"stopped"}));
        }
    });
    Ok(())
}
