include!(concat!(env!("OUT_DIR"), "/languages.rs"));

mod export;
use export::{ExportFormat, ExportOptions, LottieInfo, ExportedFrame};

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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: Theme,
    pub lang: Language,
    #[serde(default = "default_canvas_background_color")]
    pub canvas_background_color: String,
    #[serde(default = "default_canvas_background_color_dark")]
    pub canvas_background_color_dark: String,
}

fn default_canvas_background_color() -> String {
    "#FFFFFF".to_string()
}

fn default_canvas_background_color_dark() -> String {
    "#0F1115".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            lang: Language::default(),
            canvas_background_color: default_canvas_background_color(),
            canvas_background_color_dark: default_canvas_background_color_dark(),
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

#[tauri::command]
async fn export_animation(
    app: tauri::AppHandle,
    file_path: String,
    format: ExportFormat,
    options: ExportOptions,
) -> Result<bool, String> { // Changed return type to bool (success/failure)
    use ffmpeg_sidecar::download::auto_download;

    let _ = auto_download().map_err(|e| e.to_string())?;

    let content = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let json_str = String::from_utf8(content).map_err(|e| e.to_string())?;

    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Invalid Lottie JSON: {}", e))?;

    let width = json_value["w"].as_u64().unwrap_or(800) as u32;
    let height = json_value["h"].as_u64().unwrap_or(600) as u32;
    let frame_rate = json_value["fr"].as_f64().unwrap_or(30.0);
    let op = json_value["op"].as_f64().unwrap_or(100.0);

    let lottie_info = LottieInfo {
        width,
        height,
        frame_rate,
        total_frames: op,
    };

    let target_height = (options.width as f64 / width as f64 * height as f64) as u32;

    let rust_format = match format {
        ExportFormat::Gif => export::ExportFormat::Gif,
        ExportFormat::Mp4 => export::ExportFormat::Mp4,
    };

    let (output_data, extension) = export::render_to_format(
        &file_path,
        rust_format,
        export::ExportOptions {
            width: options.width,
            fps: options.fps,
            loop_gif: options.loop_gif,
            quality: options.quality,
            transparent: options.transparent,
        },
        |frame_idx| render_lottie_frame(&json_value, frame_idx, options.width, target_height),
        || lottie_info.clone(),
    )?;

    // Show file save dialog and save the file
    use tauri_plugin_dialog::DialogExt;

    let file_path_result = match format {
        ExportFormat::Gif => {
            app.dialog()
                .file()
                .add_filter("GIF Image", &["gif"])
                .set_file_name(&format!("animation.{}", extension))
                .blocking_save_file()
        },
        ExportFormat::Mp4 => {
            app.dialog()
                .file()
                .add_filter("MP4 Video", &["mp4"])
                .set_file_name(&format!("animation.{}", extension))
                .blocking_save_file()
        }
    };

    if let Some(save_path) = file_path_result {
        let path_buf = save_path.into_path().map_err(|e| format!("Failed to convert path: {:?}", e))?;
        std::fs::write(&path_buf, output_data).map_err(|e| format!("Failed to save file: {}", e))?;
        Ok(true)
    } else {
        // User cancelled the dialog
        Ok(false)
    }
}

#[tauri::command]
async fn encode_frames(
    format: String,
    frames: Vec<String>,
    _width: u32,
    _fps: u32,
    quality: u8,
    transparent: bool,
) -> Result<(Vec<u8>, String), String> {
    use ffmpeg_sidecar::command::FfmpegCommand;
    use ffmpeg_sidecar::download::auto_download;
    use std::io::{Read, Write};
    use tempfile::TempDir;

    let _ = auto_download().map_err(|e| e.to_string())?;

    let temp_dir = TempDir::with_prefix("animaview-frames-").map_err(|e| e.to_string())?;
    let temp_dir_path = temp_dir.path();

    let frame_paths: Vec<std::path::PathBuf> = frames
        .iter()
        .enumerate()
        .map(|(i, base64)| {
            let frame_path = temp_dir_path.join(format!("frame{:04}.png", i));
            let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64)
                .map_err(|e| format!("Failed to decode frame: {}", e))?;
            let mut file = std::fs::File::create(&frame_path)
                .map_err(|e| format!("Failed to create frame file: {}", e))?;
            file.write_all(&data)
                .map_err(|e| format!("Failed to write frame: {}", e))?;
            Ok(frame_path)
        })
        .collect::<Result<Vec<_>, String>>()?;

    let output_filename = match format.to_uppercase().as_str() {
        "GIF" => "animation.gif",
        "MP4" => "animation.mp4",
        _ => return Err("Invalid format".to_string()),
    };

    let output_path = temp_dir_path.join(output_filename);

    let crf = match quality {
        0..=30 => 18,
        31..=60 => 23,
        61..=100 => 28,
        _ => 23,
    };

    let mut cmd = FfmpegCommand::new();

    // Use the frame_paths variable to verify all frames were created
    if frame_paths.is_empty() {
        return Err("No frames were created".to_string());
    }

    // Use glob pattern to input all frames
    cmd.hide_banner()
        .overwrite()
        .input(temp_dir_path.join("frame%04d.png").to_str().unwrap())
        .output(output_path.to_str().unwrap());

    match format.to_uppercase().as_str() {
        "GIF" => {
            cmd.format("gif");
            if transparent {
                cmd.pix_fmt("rgba"); // Use RGBA for transparency
            } else {
                cmd.pix_fmt("rgb24"); // Use RGB without alpha channel
            }
        }
        "MP4" => {
            cmd.format("mp4")
                .codec_video("libx264")
                .preset("fast")
                .crf(crf)
                .pix_fmt("yuv420p"); // MP4 doesn't support transparency in standard format
        }
        _ => return Err("Invalid format".to_string()),
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn FFmpeg: {}", e))?;

    let status = child
        .wait()
        .map_err(|e| format!("FFmpeg process failed: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg exited with error: {:?}", status.code()));
    }

    let mut output_file =
        std::fs::File::open(&output_path).map_err(|e| format!("Failed to open output: {}", e))?;

    let mut output_data = Vec::new();
    output_file
        .read_to_end(&mut output_data)
        .map_err(|e| format!("Failed to read output: {}", e))?;

    let extension = match format.to_uppercase().as_str() {
        "GIF" => "gif",
        "MP4" => "mp4",
        _ => return Err("Invalid format".to_string()),
    };

    Ok((output_data, extension.to_string()))
}

fn render_lottie_frame(
    json: &serde_json::Value,
    frame: usize,
    _target_width: u32,
    _target_height: u32,
) -> Result<ExportedFrame, String> {
    let width = json["w"].as_u64().unwrap_or(800) as usize;
    let height = json["h"].as_u64().unwrap_or(600) as usize;

    // Initialize pixels with transparent background (alpha = 0)
    let mut pixels = vec![0u8; width * height * 4];
    for i in (0..pixels.len()).step_by(4) {
        pixels[i + 3] = 0; // Set alpha to 0 for transparency
    }

    let empty_layers: Vec<serde_json::Value> = vec![];
    let layers = json["layers"].as_array().unwrap_or(&empty_layers);

    for layer in layers {
        if let Some(is_hidden) = layer["hd"].as_bool() {
            if is_hidden {
                continue;
            }
        }

        let layer_type = layer["ty"].as_u64().unwrap_or(0);

        let in_point = layer["ip"].as_f64().unwrap_or(0.0);
        let out_point = layer["op"].as_f64().unwrap_or(100.0);

        if (frame as f64) < in_point || (frame as f64) >= out_point {
            continue;
        }

        match layer_type {
            1 => {
                render_solid_layer(layer, &mut pixels)?;
            }
            4 => {
                render_shape_layer(json, layer, frame, &mut pixels)?;
            }
            _ => {}
        }
    }

    Ok(ExportedFrame {
        width,
        height,
        pixels,
    })
}

fn render_solid_layer(
    layer: &serde_json::Value,
    pixels: &mut [u8],
) -> Result<(), String> {
    let hex_color = layer["sc"].as_str().unwrap_or("#FFFFFF");
    let color = hex_to_rgba(hex_color);

    let width = layer["sw"].as_u64().unwrap_or(800) as usize;
    let height = layer["sh"].as_u64().unwrap_or(600) as usize;

    for y in 0..height.min(pixels.len() / 4 / width) {
        for x in 0..width.min(pixels.len() / 4 / height) {
            let idx = (y * width + x) * 4;
            if idx + 3 < pixels.len() {
                pixels[idx] = color.0;
                pixels[idx + 1] = color.1;
                pixels[idx + 2] = color.2;
                pixels[idx + 3] = color.3; // Use the alpha value from the color
            }
        }
    }

    Ok(())
}

fn render_shape_layer(
    json: &serde_json::Value,
    layer: &serde_json::Value,
    frame: usize,
    pixels: &mut [u8],
) -> Result<(), String> {
    let empty_vec: Vec<serde_json::Value> = vec![];
    let shapes = layer["shapes"].as_array().unwrap_or(&empty_vec);

    for shape in shapes {
        let shape_type = shape["ty"].as_u64().unwrap_or(0);

        match shape_type {
            4 => {
                render_group(json, shape, frame, pixels)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn render_group(
    json: &serde_json::Value,
    group: &serde_json::Value,
    frame: usize,
    pixels: &mut [u8],
) -> Result<(), String> {
    let empty_vec: Vec<serde_json::Value> = vec![];
    let shapes = group["it"].as_array().unwrap_or(&empty_vec);

    for shape in shapes {
        let shape_type = shape["ty"].as_u64().unwrap_or(0);

        match shape_type {
            4 => {
                render_group(json, shape, frame, pixels)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn hex_to_rgba(hex: &str) -> (u8, u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    let a = if hex.len() > 6 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };
    (r, g, b, a)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Emitter;
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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
            read_file_content,
            get_config,
            set_config,
            export_animation,
            encode_frames
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
