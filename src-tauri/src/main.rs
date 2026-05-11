// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
fn configure_linux_webview_environment() {
    // Avoid WebKitGTK DMA-BUF renderer crashes on some Linux GPU/driver stacks.
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    // Prefer native Wayland when it is available, with X11/XWayland as a
    // fallback for sessions or WebKitGTK builds that need it.
    if std::env::var_os("GDK_BACKEND").is_none() {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            std::env::set_var("GDK_BACKEND", "wayland,x11");
        } else if std::env::var_os("DISPLAY").is_some() {
            std::env::set_var("GDK_BACKEND", "x11,wayland");
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn configure_linux_webview_environment() {}

fn main() {
    configure_linux_webview_environment();
    animaview_lib::run()
}
