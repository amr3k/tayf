use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::state::{ActiveModal, AppState};
use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_header(
    state: &AppState,
    theme: &ThemeColors,
    window: &Window,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let is_rtl = crate::i18n::is_rtl();
    let has_file = state.animation.is_some();
    let is_maximized = window.is_maximized();

    let display_title = if let Some(anim) = &state.animation {
        anim.metadata.file_name.clone()
    } else {
        "AnimaView".to_string()
    };

    div()
        .id("custom-top-bar")
        .w_full()
        .h(px(40.0))
        .flex()
        .items_center()
        .justify_between()
        .when(is_rtl, |s| s.flex_row_reverse())
        .px_2()
        .bg(theme.surface)
        .when(!is_maximized, |s| s.rounded_t_2xl())
        .border_b_1()
        .border_color(theme.border)
        .window_control_area(WindowControlArea::Drag)
        .on_mouse_down(
            MouseButton::Left,
            |e: &MouseDownEvent, window: &mut Window, _| {
                if e.click_count == 2 {
                    #[cfg(target_os = "macos")]
                    window.titlebar_double_click();
                    #[cfg(not(target_os = "macos"))]
                    window.zoom_window();
                } else {
                    window.start_window_move();
                }
            },
        )
        .on_mouse_down(
            MouseButton::Right,
            |e: &MouseDownEvent, window: &mut Window, _| {
                window.show_window_menu(e.position);
            },
        )
        // Left / Start section: App Brand / Menu + Sidebar Toggle
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .when(is_rtl, |s| s.flex_row_reverse())
                // App Logo / Menu Trigger
                .child(render_app_menu(state, theme, is_rtl, cx))
                // Sidebar toggle button (if a file is loaded)
                .children(if has_file {
                    Some(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .when_else(is_rtl, |s| s.mr_2(), |s| s.ml_2())
                            .when(is_rtl, |s| s.flex_row_reverse())
                            .child(
                                render_header_btn(
                                    "toggle-sidebar-btn",
                                    crate::ui::icon::Icon::SidebarRight,
                                    theme,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.is_app_menu_open = false;
                                        this.state.is_sidebar_open = !this.state.is_sidebar_open;
                                        cx.notify();
                                    }),
                                ),
                            ),
                    )
                } else {
                    None
                }),
        )
        // Center section: Title / File Name (Draggable)
        .child(
            div()
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .px_4()
                .overflow_hidden()
                .window_control_area(WindowControlArea::Drag)
                .on_mouse_down(
                    MouseButton::Left,
                    |e: &MouseDownEvent, window: &mut Window, _| {
                        if e.click_count == 2 {
                            #[cfg(target_os = "macos")]
                            window.titlebar_double_click();
                            #[cfg(not(target_os = "macos"))]
                            window.zoom_window();
                        } else {
                            window.start_window_move();
                        }
                    },
                )
                .on_mouse_down(
                    MouseButton::Right,
                    |e: &MouseDownEvent, window: &mut Window, _| {
                        window.show_window_menu(e.position);
                    },
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if has_file { theme.text_primary } else { theme.text_muted })
                        .overflow_hidden()
                        .child(display_title),
                ),
        )
        // Right / End section: Window Controls
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .when(is_rtl, |s| s.flex_row_reverse())
                .child(render_window_btn(
                    "window-minimize-btn",
                    crate::ui::icon::Icon::WindowMinimize,
                    WindowControlArea::Min,
                    theme,
                    |_, window, _| window.minimize_window(),
                ))
                .child(render_window_btn(
                    "window-maximize-btn",
                    if is_maximized {
                        crate::ui::icon::Icon::WindowRestore
                    } else {
                        crate::ui::icon::Icon::WindowMaximize
                    },
                    WindowControlArea::Max,
                    theme,
                    |_, window, _| window.zoom_window(),
                ))
                .child(render_window_close_btn(
                    "window-close-btn",
                    crate::ui::icon::Icon::WindowClose,
                    theme,
                    |_, window, _| window.remove_window(),
                )),
        )
        .into_any_element()
}

fn render_app_menu(
    state: &AppState,
    theme: &ThemeColors,
    is_rtl: bool,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let is_open = state.is_app_menu_open;

    div()
        .id("app-menu-container")
        .relative()
        .child(
            div()
                .id("app-menu-trigger")
                .group("app-menu-btn")
                .h(px(28.0))
                .px_2()
                .gap_1p5()
                .rounded_md()
                .flex()
                .items_center()
                .justify_center()
                .when(is_rtl, |s| s.flex_row_reverse())
                .cursor_pointer()
                .bg(if is_open { theme.surface_active } else { theme.surface })
                .border_1()
                .border_color(if is_open { theme.accent } else { theme.border })
                .hover(|s| s.bg(theme.surface_hover).border_color(theme.accent))
                .active(|s| s.bg(theme.surface_active))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.state.is_app_menu_open = !this.state.is_app_menu_open;
                        cx.notify();
                    }),
                )
                // App Icon
                .child(
                    div()
                        .size(px(20.0))
                        .rounded_md()
                        .bg(theme.accent)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.accent_text)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .child("A"),
                )
                // Default Title Text (hidden when open or on hover)
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("AnimaView"),
                )
                // Dropdown indicator arrow
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.accent)
                        .hidden()
                        .when(is_open, |s| s.flex())
                        .group_hover("app-menu-btn", |s| s.flex())
                        .flex()
                        .items_center()
                        .gap_1()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .child(
                            crate::ui::icon::render_icon(crate::ui::icon::Icon::ArrowDown)
                                .size(px(10.0))
                                .text_color(theme.accent),
                        ),
                ),
        )
        .children(if is_open {
            Some(
                deferred(
                    div()
                        .id("app-dropdown-menu")
                        .absolute()
                        .top(px(32.0))
                        .when_else(is_rtl, |s| s.right_0(), |s| s.left_0())
                        .w(px(210.0))
                        .p_1()
                        .rounded_xl()
                        .bg(theme.surface)
                        .border_1()
                        .border_color(theme.border)
                        .shadow_xl()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        // Open File
                        .child(render_menu_item(
                            "menu-opt-open",
                            crate::ui::icon::Icon::Folder,
                            t!("open_file").to_string(),
                            Some(if cfg!(target_os = "macos") { "⌘O" } else { "Ctrl+O" }),
                            theme,
                            is_rtl,
                            cx.listener(|this, _, _, cx| {
                                this.state.is_app_menu_open = false;
                                this.open_file_dialog(cx);
                            }),
                        ))
                        // Divider
                        .child(
                            div()
                                .h(px(1.0))
                                .my_1()
                                .mx_1()
                                .bg(theme.border),
                        )
                        // Preferences / Settings
                        .child(render_menu_item(
                            "menu-opt-preferences",
                            crate::ui::icon::Icon::Settings,
                            t!("preferences").to_string(),
                            Some(if cfg!(target_os = "macos") { "⌘P" } else { "Ctrl+P" }),
                            theme,
                            is_rtl,
                            cx.listener(|this, _, _, cx| {
                                this.state.is_app_menu_open = false;
                                this.state.active_modal = ActiveModal::Preferences;
                                cx.notify();
                            }),
                        ))
                        // About
                        .child(render_menu_item(
                            "menu-opt-about",
                            crate::ui::icon::Icon::Info,
                            t!("about").to_string(),
                            Some("F1"),
                            theme,
                            is_rtl,
                            cx.listener(|this, _, _, cx| {
                                this.state.is_app_menu_open = false;
                                this.state.active_modal = ActiveModal::About;
                                cx.notify();
                            }),
                        ))
                        // Divider
                        .child(
                            div()
                                .h(px(1.0))
                                .my_1()
                                .mx_1()
                                .bg(theme.border),
                        )
                        // Quit
                        .child(render_menu_item(
                            "menu-opt-quit",
                            crate::ui::icon::Icon::WindowClose,
                            t!("quit").to_string(),
                            Some(if cfg!(target_os = "macos") { "⌘Q" } else { "Alt+F4" }),
                            theme,
                            is_rtl,
                            cx.listener(|_, _, _, cx| {
                                cx.quit();
                            }),
                        )),
                )
            )
        } else {
            None
        })
}

fn render_menu_item(
    id: &'static str,
    icon: crate::ui::icon::Icon,
    label: String,
    shortcut: Option<&'static str>,
    theme: &ThemeColors,
    is_rtl: bool,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .w_full()
        .h(px(30.0))
        .px_2p5()
        .rounded_lg()
        .flex()
        .items_center()
        .justify_between()
        .when(is_rtl, |s| s.flex_row_reverse())
        .text_xs()
        .text_color(theme.text_primary)
        .bg(theme.surface)
        .hover(|s| s.bg(theme.surface_hover))
        .active(|s| s.bg(theme.surface_active))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap_2()
                .when(is_rtl, |s| s.flex_row_reverse())
                .child(
                    crate::ui::icon::render_icon(icon)
                        .size(px(14.0))
                        .text_color(theme.text_secondary),
                )
                .child(
                    div()
                        .font_weight(FontWeight::NORMAL)
                        .child(label),
                ),
        )
        .children(if let Some(sc) = shortcut {
            Some(
                div()
                    .text_xs()
                    .font_weight(FontWeight::LIGHT)
                    .text_color(theme.text_muted)
                    .child(sc),
            )
        } else {
            None
        })
}

fn render_header_btn(
    id: &'static str,
    icon: crate::ui::icon::Icon,
    theme: &ThemeColors,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(28.0))
        .rounded_md()
        .flex()
        .items_center()
        .justify_center()
        .text_color(theme.text_secondary)
        .bg(theme.surface)
        .hover(|s| s.bg(theme.surface_hover).text_color(theme.text_primary))
        .active(|s| s.bg(theme.surface_active))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .child(
            crate::ui::icon::render_icon(icon)
                .size(px(15.0))
                .text_color(theme.text_secondary),
        )
}

fn render_window_btn(
    id: &'static str,
    icon: crate::ui::icon::Icon,
    area: WindowControlArea,
    theme: &ThemeColors,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(28.0))
        .rounded_md()
        .flex()
        .items_center()
        .justify_center()
        .text_color(theme.text_secondary)
        .bg(theme.surface)
        .hover(|s| s.bg(theme.surface_hover).text_color(theme.text_primary))
        .active(|s| s.bg(theme.surface_active))
        .cursor_pointer()
        .window_control_area(area)
        .on_mouse_down(MouseButton::Left, handler)
        .child(
            crate::ui::icon::render_icon(icon)
                .size(px(14.0))
                .text_color(theme.text_secondary),
        )
}

fn render_window_close_btn(
    id: &'static str,
    icon: crate::ui::icon::Icon,
    theme: &ThemeColors,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(28.0))
        .rounded_md()
        .flex()
        .items_center()
        .justify_center()
        .text_color(theme.text_secondary)
        .bg(theme.surface)
        .hover(|s| s.bg(rgb(0xe05252)).text_color(rgb(0xffffff)))
        .active(|s| s.bg(rgb(0xc53939)).text_color(rgb(0xffffff)))
        .cursor_pointer()
        .window_control_area(WindowControlArea::Close)
        .on_mouse_down(MouseButton::Left, handler)
        .child(
            crate::ui::icon::render_icon(icon)
                .size(px(14.0))
                .text_color(theme.text_secondary),
        )
}
