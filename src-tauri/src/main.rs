// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Fix for "Failed to create GBM buffer" on Linux
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    animaview_lib::run()
}
