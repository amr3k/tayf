// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn read_file_content(file_path: String) -> Result<Vec<u8>, String> {
    std::fs::read(file_path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Emitter;
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .menu(|handle| {
            let file_menu = tauri::menu::Submenu::with_items(
                handle,
                "File",
                true,
                &[
                    &tauri::menu::MenuItem::with_id(handle, "open", "Open", true, Some("Ctrl+O"))?,
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
            "exit" => {
                app.exit(0);
            }
            "about" => {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("about") {
                    let _ = window.set_focus();
                } else {
                    let _ = tauri::WebviewWindowBuilder::new(
                        app,
                        "about",
                        tauri::WebviewUrl::App("about".into()),
                    )
                    .title("About AnimaView")
                    .inner_size(400.0, 500.0)
                    .resizable(false)
                    .center()
                    .build();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![greet, read_file_content])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
