mod chat;
use tauri::Manager;
mod desktop;
mod gateway;
mod voice;
pub fn run() {
    tauri::Builder::default()
        .manage(chat::ChatState::default())
        .manage(voice::VoiceState::default())
        .manage(desktop::DesktopSettings::default())
        .setup(desktop::setup)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                voice::shutdown(window.app_handle());
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            gateway::check_connection,
            gateway::call_capability,
            desktop::desktop_control,
            voice::speak_text,
            voice::stop_speech,
            voice::speech_status,
            voice::voice_input,
            chat::configure_chat,
            chat::chat_reply,
            chat::gpu_speech
        ])
        .run(tauri::generate_context!())
        .expect("error running app");
}
