use serde::Serialize;
use tauri::{LogicalSize, Manager};
#[derive(Serialize)]
pub struct DesktopState {
    visible: bool,
    always_on_top: bool,
    click_through: bool,
    scale: u32,
}
#[derive(Default)]
pub struct DesktopSettings(pub std::sync::Mutex<(bool, u32)>);
#[tauri::command]
pub fn desktop_control(
    app: tauri::AppHandle,
    action: String,
    value: Option<u32>,
    state: tauri::State<DesktopSettings>,
) -> Result<DesktopState, String> {
    let window = app.get_webview_window("pet").ok_or("悬浮窗尚未就绪")?;
    let mut settings = state.0.lock().map_err(|_| "窗口状态不可用")?;
    let result = match action.as_str() {
        "show" => window.show(),
        "hide" => {
            crate::voice::shutdown(&app);
            window.hide()
        }
        "drag" => window.start_dragging(),
        "top" => window.set_always_on_top(value.unwrap_or(1) != 0),
        "through" => {
            let enabled = value.unwrap_or(0) != 0;
            let result = window.set_ignore_cursor_events(enabled);
            if result.is_ok() {
                settings.0 = enabled;
            }
            result
        }
        "scale" => {
            let scale = value.unwrap_or(100);
            if !(60..=160).contains(&scale) {
                return Err("缩放必须在 60% 至 160% 之间".into());
            }
            let result = window.set_size(LogicalSize::new(
                260.0 * scale as f64 / 100.0,
                300.0 * scale as f64 / 100.0,
            ));
            if result.is_ok() {
                settings.1 = scale;
            }
            result
        }
        "console" => {
            let main = app.get_webview_window("main").ok_or("控制台不可用")?;
            main.show().and_then(|_| main.set_focus())
        }
        "state" => Ok(()),
        _ => return Err("不支持的桌面操作".into()),
    };
    result.map_err(|e| e.to_string())?;
    Ok(DesktopState {
        visible: window.is_visible().map_err(|e| e.to_string())?,
        always_on_top: window.is_always_on_top().map_err(|e| e.to_string())?,
        click_through: settings.0,
        scale: if settings.1 == 0 { 100 } else { settings.1 },
    })
}
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::{
        menu::{Menu, MenuItem},
        tray::TrayIconBuilder,
    };
    let restore = MenuItem::with_id(app, "restore", "恢复悬浮窗（关闭穿透）", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏悬浮窗", true, None::<&str>)?;
    let console = MenuItem::with_id(app, "console", "打开控制台", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出助手", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&restore, &hide, &console, &quit])?;
    let pixels = [67u8, 103, 79, 255].repeat(32 * 32);
    TrayIconBuilder::new()
        .icon(tauri::image::Image::new_owned(pixels, 32, 32))
        .tooltip("Agent PC Assistant")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "restore" => {
                if let Some(w) = app.get_webview_window("pet") {
                    let _ = w.set_ignore_cursor_events(false);
                    let _ = w.show();
                    let _ = w.set_focus();
                    if let Ok(mut state) = app.state::<DesktopSettings>().0.lock() {
                        state.0 = false;
                    }
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("pet") {
                    let _ = w.hide();
                }
            }
            "console" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => {
                crate::voice::shutdown(app);
                app.exit(0)
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}
