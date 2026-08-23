use std::borrow::Cow;
use anyhow::Result;
use gpui::prelude::*;
use gpui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Play,
    Pause,
    Previous,
    Next,
    Repeat,
    Folder,
    Upload,
    Download,
    Settings,
    SidebarRight,
    Info,
    Cancel,
    Sun,
    Moon,
    Computer,
    ArrowDown,
    Tick,
    ArrowUpRight,
    PaintBoard,
    Film,
    Image,
    WindowMinimize,
    WindowMaximize,
    WindowRestore,
    WindowClose,
    Github,
    Globe,
}

pub const APP_LOGO: &str = "brand/logo.png";

const APP_LOGO_PNG: &[u8] = include_bytes!("../../resources/icons/icon.png");

impl Icon {
    pub fn path(self) -> &'static str {
        match self {
            Icon::Play => "icons/play.svg",
            Icon::Pause => "icons/pause.svg",
            Icon::Previous => "icons/previous.svg",
            Icon::Next => "icons/next.svg",
            Icon::Repeat => "icons/repeat.svg",
            Icon::Folder => "icons/folder-01.svg",
            Icon::Upload => "icons/upload-04.svg",
            Icon::Download => "icons/download-01.svg",
            Icon::Settings => "icons/settings-02.svg",
            Icon::SidebarRight => "icons/sidebar-right.svg",
            Icon::Info => "icons/information-circle.svg",
            Icon::Cancel => "icons/cancel-01.svg",
            Icon::Sun => "icons/sun-01.svg",
            Icon::Moon => "icons/moon-02.svg",
            Icon::Computer => "icons/computer.svg",
            Icon::ArrowDown => "icons/arrow-down-01.svg",
            Icon::Tick => "icons/tick-02.svg",
            Icon::ArrowUpRight => "icons/arrow-up-right-01.svg",
            Icon::PaintBoard => "icons/paint-board.svg",
            Icon::Film => "icons/film-02.svg",
            Icon::Image => "icons/image-01.svg",
            Icon::WindowMinimize => "icons/window-minimize.svg",
            Icon::WindowMaximize => "icons/window-maximize.svg",
            Icon::WindowRestore => "icons/window-restore.svg",
            Icon::WindowClose => "icons/window-close.svg",
            Icon::Github => "icons/github.svg",
            Icon::Globe => "icons/globe.svg",
        }
    }
}

pub struct IconAssets;

impl AssetSource for IconAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == APP_LOGO {
            return Ok(Some(Cow::Borrowed(APP_LOGO_PNG)));
        }

        let svg_data = match path {
            "icons/play.svg" => Some(PLAY_SVG),
            "icons/pause.svg" => Some(PAUSE_SVG),
            "icons/previous.svg" => Some(PREVIOUS_SVG),
            "icons/next.svg" => Some(NEXT_SVG),
            "icons/repeat.svg" => Some(REPEAT_SVG),
            "icons/folder-01.svg" => Some(FOLDER_SVG),
            "icons/upload-04.svg" => Some(UPLOAD_SVG),
            "icons/download-01.svg" => Some(DOWNLOAD_SVG),
            "icons/settings-02.svg" => Some(SETTINGS_SVG),
            "icons/sidebar-right.svg" => Some(SIDEBAR_RIGHT_SVG),
            "icons/information-circle.svg" => Some(INFO_SVG),
            "icons/cancel-01.svg" => Some(CANCEL_SVG),
            "icons/sun-01.svg" => Some(SUN_SVG),
            "icons/moon-02.svg" => Some(MOON_SVG),
            "icons/computer.svg" => Some(COMPUTER_SVG),
            "icons/arrow-down-01.svg" => Some(ARROW_DOWN_SVG),
            "icons/tick-02.svg" => Some(TICK_SVG),
            "icons/arrow-up-right-01.svg" => Some(ARROW_UP_RIGHT_SVG),
            "icons/paint-board.svg" => Some(PAINT_BOARD_SVG),
            "icons/film-02.svg" => Some(FILM_SVG),
            "icons/image-01.svg" => Some(IMAGE_SVG),
            "icons/window-minimize.svg" => Some(WINDOW_MINIMIZE_SVG),
            "icons/window-maximize.svg" => Some(WINDOW_MAXIMIZE_SVG),
            "icons/window-restore.svg" => Some(WINDOW_RESTORE_SVG),
            "icons/window-close.svg" => Some(WINDOW_CLOSE_SVG),
            "icons/github.svg" => Some(GITHUB_SVG),
            "icons/globe.svg" => Some(GLOBE_SVG),
            _ => None,
        };

        Ok(svg_data.map(|s| Cow::Borrowed(s.as_bytes())))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        let assets = vec![
            APP_LOGO.into(),
            "icons/play.svg".into(),
            "icons/pause.svg".into(),
            "icons/previous.svg".into(),
            "icons/next.svg".into(),
            "icons/repeat.svg".into(),
            "icons/folder-01.svg".into(),
            "icons/upload-04.svg".into(),
            "icons/download-01.svg".into(),
            "icons/settings-02.svg".into(),
            "icons/sidebar-right.svg".into(),
            "icons/information-circle.svg".into(),
            "icons/cancel-01.svg".into(),
            "icons/sun-01.svg".into(),
            "icons/moon-02.svg".into(),
            "icons/computer.svg".into(),
            "icons/arrow-down-01.svg".into(),
            "icons/tick-02.svg".into(),
            "icons/arrow-up-right-01.svg".into(),
            "icons/paint-board.svg".into(),
            "icons/film-02.svg".into(),
            "icons/image-01.svg".into(),
            "icons/window-minimize.svg".into(),
            "icons/window-maximize.svg".into(),
            "icons/window-restore.svg".into(),
            "icons/window-close.svg".into(),
            "icons/github.svg".into(),
            "icons/globe.svg".into(),
        ];
        Ok(assets)
    }
}

pub fn render_icon(icon: Icon) -> Svg {
    svg()
        .path(icon.path())
        .flex_shrink_0()
}

// Huge Icons SVGs (Stroke 1.5)
const PLAY_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linejoin="round" stroke-width="1.5" d="M18.89 12.846c-.353 1.343-2.023 2.292-5.364 4.19c-3.23 1.835-4.845 2.752-6.146 2.384a3.25 3.25 0 0 1-1.424-.841C5 17.614 5 15.743 5 12s0-5.614.956-6.579a3.25 3.25 0 0 1 1.424-.84c1.301-.37 2.916.548 6.146 2.383c3.34 1.898 5.011 2.847 5.365 4.19a3.3 3.3 0 0 1 0 1.692Z"/></svg>"#;

const PAUSE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-width="1.5" d="M4 7c0-1.414 0-2.121.44-2.56C4.878 4 5.585 4 7 4s2.121 0 2.56.44C10 4.878 10 5.585 10 7v10c0 1.414 0 2.121-.44 2.56C9.122 20 8.415 20 7 20s-2.121 0-2.56-.44C4 19.122 4 18.415 4 17zm10 0c0-1.414 0-2.121.44-2.56C14.878 4 15.585 4 17 4s2.121 0 2.56.44C20 4.878 20 5.585 20 7v10c0 1.414 0 2.121-.44 2.56c-.439.44-1.146.44-2.56.44s-2.121 0-2.56-.44C14 19.122 14 18.415 14 17z"/></svg>"#;

const PREVIOUS_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><path stroke-linejoin="round" d="M8.065 12.626c.254 1.211 1.608 2.082 4.315 3.822c2.945 1.893 4.417 2.84 5.61 2.475c.403-.124.775-.34 1.088-.635C20 17.418 20 15.612 20 12s0-5.418-.922-6.288a2.8 2.8 0 0 0-1.088-.635c-1.193-.365-2.665.582-5.61 2.475c-2.707 1.74-4.06 2.61-4.315 3.822c-.087.412-.087.84 0 1.252Z"/><path stroke-linecap="round" d="M4 4v16"/></g></svg>"#;

const NEXT_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><path stroke-linejoin="round" d="M15.935 12.626c-.254 1.211-1.608 2.082-4.315 3.822c-2.945 1.893-4.417 2.84-5.61 2.475a2.8 2.8 0 0 1-1.088-.635C4 17.418 4 15.612 4 12s0-5.418.922-6.288a2.8 2.8 0 0 1 1.089-.635c1.192-.365 2.664.582 5.609 2.475c2.707 1.74 4.06 2.61 4.315 3.822c.087.412.087.84 0 1.252Z"/><path stroke-linecap="round" d="M20 5v14"/></g></svg>"#;

const REPEAT_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="m16.388 3l1.003.976c.448.436.672.654.593.839C17.906 5 17.59 5 16.955 5h-7.76C5.22 5 2 8.134 2 12c0 1.487.477 2.866 1.29 4m4.322 5l-1.003-.976c-.448-.436-.672-.654-.593-.839C6.094 19 6.41 19 7.045 19h7.76C18.78 19 22 15.866 22 12a6.84 6.84 0 0 0-1.29-4"/></svg>"#;

const FOLDER_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-width="1.5" d="M8 7h8.75c2.107 0 3.16 0 3.917.506a3 3 0 0 1 .827.827C22 9.09 22 10.143 22 12.25c0 3.511 0 5.267-.843 6.528a5 5 0 0 1-1.38 1.38C18.518 21 16.762 21 13.25 21H12c-4.714 0-7.071 0-8.536-1.465C2 18.072 2 15.715 2 11V7.944c0-1.816 0-2.724.38-3.406A3 3 0 0 1 3.538 3.38C4.22 3 5.128 3 6.944 3C8.108 3 8.69 3 9.2 3.191c1.163.436 1.643 1.493 2.168 2.542L12 7"/></svg>"#;

const UPLOAD_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5.25 21h13.5c.232 0 .348 0 .446-.01a2 2 0 0 0 1.794-1.794c.01-.098.01-.214.01-.446s0-.348-.01-.446a2 2 0 0 0-1.794-1.794c-.098-.01-.214-.01-.446-.01h-1.69c-.228 0-.342 0-.451.012a2 2 0 0 0-1.03.427c-.087.069-.168.15-.329.311a4 4 0 0 1-.328.311a2 2 0 0 1-1.03.427c-.11.012-.224.012-.453.012h-2.878c-.229 0-.343 0-.452-.012a2 2 0 0 1-1.03-.427c-.087-.069-.168-.15-.329-.311s-.242-.242-.328-.311a2 2 0 0 0-1.03-.427c-.11-.012-.224-.012-.453-.012H5.25c-.232 0-.348 0-.446.01a2 2 0 0 0-1.794 1.794c-.01.098-.01.214-.01.446s0 .348.01.446a2 2 0 0 0 1.794 1.794c.098.01.214.01.446.01M16.5 7.5S13.186 3 12 3S7.5 7.5 7.5 7.5M12 4v11"/></svg>"#;

const DOWNLOAD_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3 17c0 .93 0 1.395.102 1.777a3 3 0 0 0 2.121 2.121C5.605 21 6.07 21 7 21h10c.93 0 1.395 0 1.776-.102a3 3 0 0 0 2.122-2.121C21 18.395 21 17.93 21 17m-4.5-5.5S13.186 16 12 16s-4.5-4.5-4.5-4.5M12 15V3"/></svg>"#;

const SETTINGS_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M15.5 12a3.5 3.5 0 1 1-7 0a3.5 3.5 0 0 1 7 0Z"/><path stroke-linecap="round" d="M21.011 14.097c.522-.141.783-.212.886-.346c.103-.135.103-.351.103-.784v-1.934c0-.433 0-.65-.103-.784s-.364-.205-.886-.345c-1.95-.526-3.171-2.565-2.668-4.503c.139-.533.208-.8.142-.956s-.256-.264-.635-.479l-1.725-.98c-.372-.21-.558-.316-.725-.294s-.356.21-.733.587c-1.459 1.455-3.873 1.455-5.333 0c-.377-.376-.565-.564-.732-.587c-.167-.022-.353.083-.725.295l-1.725.979c-.38.215-.57.323-.635.48c-.066.155.003.422.141.955c.503 1.938-.718 3.977-2.669 4.503c-.522.14-.783.21-.886.345S2 10.6 2 11.033v1.934c0 .433 0 .65.103.784s.364.205.886.346c1.95.526 3.171 2.565 2.668 4.502c-.139.533-.208.8-.142.956s.256.264.635.48l1.725.978c.372.212.558.317.725.295s.356-.21.733-.587c1.46-1.457 3.876-1.457 5.336 0c.377.376.565.564.732.587c.167.022.353-.083.726-.295l1.724-.979c.38-.215.57-.323.635-.48s-.003-.422-.141-.955c-.504-1.937.716-3.976 2.666-4.502Z"/></g></svg>"#;

const SIDEBAR_RIGHT_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2 12c0-3.69 0-5.534.814-6.841a4.8 4.8 0 0 1 1.105-1.243C5.08 3 6.72 3 10 3h4c3.28 0 4.919 0 6.081.916c.43.338.804.759 1.105 1.243C22 6.466 22 8.31 22 12s0 5.534-.814 6.841a4.8 4.8 0 0 1-1.105 1.243C18.92 21 17.28 21 14 21h-4c-3.28 0-4.919 0-6.081-.916a4.8 4.8 0 0 1-1.105-1.243C2 17.534 2 15.69 2 12Z"/><path stroke-linejoin="round" d="M14.5 3v18"/><path stroke-linecap="round" stroke-linejoin="round" d="M18 7h1m-1 3h1"/></g></svg>"#;

const INFO_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4m.125-3.75H12m.25 0a.25.25 0 1 0-.5 0a.25.25 0 0 0 .5 0"/></g></svg>"#;

const CANCEL_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M18 6L6 18m12 0L6 6"/></svg>"#;

const SUN_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M17 12a5 5 0 1 1-10.001 0a5 5 0 0 1 10 0m-4.874-8.75H12m.124 17.5H12m8.751-8.625V12m-17.5.125V12m15.025-6.099l-.088-.088M5.9 18.275l-.089-.088m12.287.089l.088-.089M5.724 5.901l.089-.088M12.25 3.25a.25.25 0 1 1-.5 0a.25.25 0 0 1 .5 0m0 17.5a.25.25 0 1 1-.5 0a.25.25 0 0 1 .5 0m8.5-8.5a.25.25 0 1 1 0-.5a.25.25 0 0 1 0 .5m-17.5 0a.25.25 0 1 1 0-.5a.25.25 0 0 1 0 .5m15.114-6.26a.25.25 0 1 1-.354-.354a.25.25 0 0 1 .354.353M5.989 18.362a.25.25 0 1 1-.354-.353a.25.25 0 0 1 .354.353m12.021.001a.25.25 0 1 1 .354-.354a.25.25 0 0 1-.354.354M5.636 5.99a.25.25 0 1 1 .353-.354a.25.25 0 0 1-.353.354"/></svg>"#;

const MOON_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21.5 14.078A8.557 8.557 0 0 1 9.922 2.5C5.668 3.497 2.5 7.315 2.5 11.873a9.627 9.627 0 0 0 9.627 9.627c4.558 0 8.376-3.168 9.373-7.422"/></svg>"#;

const COMPUTER_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M14 21h2m-2 0a1.5 1.5 0 0 1-1.5-1.5V17H12m2 4h-4m0 0H8m2 0a1.5 1.5 0 0 0 1.5-1.5V17h.5m0 0v4m4-18H8c-2.828 0-4.243 0-5.121.879C2 4.757 2 6.172 2 9v2c0 2.828 0 4.243.879 5.121C3.757 17 5.172 17 8 17h8c2.828 0 4.243 0 5.121-.879C22 15.243 22 13.828 22 11V9c0-2.828 0-4.243-.879-5.121C20.243 3 18.828 3 16 3"/></svg>"#;

const ARROW_DOWN_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M18 9s-4.419 6-6 6s-6-6-6-6"/></svg>"#;

const TICK_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="m5 14l3.5 3.5L19 6.5"/></svg>"#;

const ARROW_UP_RIGHT_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 6.65s6.938-.542 7.915.435S17.35 15 17.35 15m-.85-7.5l-10 10"/></svg>"#;

const PAINT_BOARD_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 12c0-5.523-4.477-10-10-10S2 6.477 2 12s4.477 10 10 10c.842 0 2 .116 2-1c0-.609-.317-1.079-.631-1.546c-.46-.683-.917-1.359-.369-2.454c.667-1.333 1.778-1.333 3.482-1.333c.851 0 1.851 0 3.018-.167c2.101-.3 2.5-1.592 2.5-3.5Z"/><circle cx="9.5" cy="8.5" r="1.5"/><circle cx="16.5" cy="9.5" r="1.5"/><path stroke-linecap="round" stroke-linejoin="round" d="M7.125 15H7m.25 0a.25.25 0 1 1-.5 0a.25.25 0 0 1 .5 0"/></g></svg>"#;

const FILM_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2.5 12c0-4.478 0-6.718 1.391-8.109S7.521 2.5 12 2.5c4.478 0 6.718 0 8.109 1.391S21.5 7.521 21.5 12c0 4.478 0 6.718-1.391 8.109S16.479 21.5 12 21.5c-4.478 0-6.718 0-8.109-1.391S2.5 16.479 2.5 12Z"/><path stroke-linejoin="round" d="M7 2.5v19m10-19v19"/><path stroke-linecap="round" stroke-linejoin="round" d="M2.5 7.5H7m10 0h4.5m-19 9H7m10 0h4.5M10.5 10l3.5 2l-3.5 2z"/></g></svg>"#;

const IMAGE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2.5 12c0-4.478 0-6.718 1.391-8.109S7.521 2.5 12 2.5c4.478 0 6.718 0 8.109 1.391S21.5 7.521 21.5 12c0 4.478 0 6.718-1.391 8.109S16.479 21.5 12 21.5c-4.478 0-6.718 0-8.109-1.391S2.5 16.479 2.5 12Z"/><circle cx="16.5" cy="7.5" r="1.5"/><path stroke-linecap="round" stroke-linejoin="round" d="m16 21l-6.293-6.293a1 1 0 0 0-1.414 0L2.5 20.5m19-2.5l-2.793-2.793a1 1 0 0 0-1.414 0L13.5 19"/></g></svg>"#;

const WINDOW_MINIMIZE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 12h14"/></svg>"#;

const WINDOW_MAXIMIZE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect width="14" height="14" x="5" y="5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" rx="2"/></svg>"#;

const WINDOW_RESTORE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><rect width="11" height="11" x="4" y="9" rx="1.5"/><path d="M8 9V6a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-3"/></g></svg>"#;

const WINDOW_CLOSE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M18 6L6 18m12 0L6 6"/></svg>"#;

const GITHUB_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M10 20.568c-3.429 1.157-6.286 0-8-3.568"/><path d="M10 22v-3.242c0-.598.184-1.118.48-1.588c.204-.322.064-.78-.303-.88C7.134 15.452 5 14.107 5 9.645c0-1.16.38-2.25 1.048-3.2c.166-.236.25-.354.27-.46c.02-.108-.015-.247-.085-.527c-.283-1.136-.264-2.343.16-3.43c0 0 .877-.287 2.874.96c.456.285.684.428.885.46s.469-.035 1.005-.169A9.5 9.5 0 0 1 13.5 3a9.6 9.6 0 0 1 2.343.28c.536.134.805.2 1.006.169c.2-.032.428-.175.884-.46c1.997-1.247 2.874-.96 2.874-.96c.424 1.087.443 2.294.16 3.43c-.07.28-.104.42-.084.526s.103.225.269.461c.668.95 1.048 2.04 1.048 3.2c0 4.462-2.134 5.807-5.177 6.643c-.367.101-.507.559-.303.88c.296.47.48.99.48 1.589V22"/></g></svg>"#;

const GLOBE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><path stroke-linejoin="round" d="M8 12c0 6 4 10 4 10s4-4 4-10s-4-10-4-10s-4 4-4 10Z"/><path stroke-linecap="round" stroke-linejoin="round" d="M21 15H3m18-6H3"/></g></svg>"#;

