// include the generated language definitions
include!(concat!(env!("OUT_DIR"), "/languages.rs"));

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn read_file_content(file_path: String) -> Result<Vec<u8>, String> {
    std::fs::read(file_path).map_err(|e| e.to_string())
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub theme: Theme,
    pub lang: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            lang: "en".to_string(),
        }
    }
}

fn get_config_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    use tauri::Manager;
    let mut config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    config_dir.push("configurations.json");
    Ok(config_dir)
}

#[tauri::command]
async fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let config_path = get_config_path(&app)?;
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }
    let content = std::fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    use tauri::Emitter;
    let config_path = get_config_path(&app)?;
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(config_path, content).map_err(|e| e.to_string())?;

    // notify all windows about the config update
    let _ = app.emit("config-updated", config);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Emitter;
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .menu(|handle| {
            let file_menu = tauri::menu::Submenu::with_items(
                handle,
                "File",
                true,
                &[
                    &tauri::menu::MenuItem::with_id(handle, "open", "Open", true, Some("Ctrl+O"))?,
                    &tauri::menu::MenuItem::with_id(
                        handle,
                        "preferences",
                        "Preferences",
                        true,
                        Some("Ctrl+P"),
                    )?,
                    &tauri::menu::MenuItem::with_id(handle, "exit", "Exit", true, Some("Ctrl+Q"))?,
                ],
            )?;

            let help_menu = tauri::menu::Submenu::with_items(
                handle,
                "Help",
                true,
                &[&tauri::menu::MenuItem::with_id(
                    handle,
                    "about",
                    "About",
                    true,
                    None::<&str>,
                )?],
            )?;

            tauri::menu::Menu::with_items(handle, &[&file_menu, &help_menu])
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                let _ = app.emit("menu-open", ());
            }
            "preferences" => {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("preferences") {
                    let _ = window.set_focus();
                } else {
                    let _ = tauri::WebviewWindowBuilder::new(
                        app,
                        "preferences",
                        tauri::WebviewUrl::App("preferences".into()),
                    )
                    .title("Preferences")
                    .inner_size(400.0, 500.0)
                    .resizable(false)
                    .center()
                    .build();
                }
            }
            "exit" => {
                app.exit(0);
            }
            "about" => {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("about") {
                    let _ = window.set_focus();
                } else {
                    if let Ok(window) = tauri::WebviewWindowBuilder::new(
                        app,
                        "about",
                        tauri::WebviewUrl::App("about".into()),
                    )
                    .title("About AnimaView")
                    .inner_size(400.0, 500.0)
                    .resizable(false)
                    .center()
                    .build()
                    {
                        let _ = window.remove_menu();
                    }
                }
            }
            _ => {}
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api: _, .. } => {
                if window.label() == "main" {
                    use tauri::Manager;
                    for (_, w) in window.app_handle().webview_windows() {
                        if w.label() != "main" {
                            let _ = w.close();
                        }
                    }
                }
            }
            _ => {}
        })
        .setup(|app| {
            use std::env;
            use tauri::Emitter;
            use tauri::Manager;

            let main_window = app.get_webview_window("main");

            if let Some(ref main_window) = main_window {
                let args: Vec<String> = env::args().collect();

                for arg in args.iter().skip(1) {
                    if arg.ends_with(".json") || arg.ends_with(".lottie") {
                        let file_path = arg.clone();
                        let _ = main_window.emit::<String>("file-opened", file_path);
                        break;
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            read_file_content,
            get_config,
            set_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
