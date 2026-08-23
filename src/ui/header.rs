use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::config::Theme;
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
        .border_b_1()
        .border_color(theme.border)
        .window_control_area(WindowControlArea::Drag)
        .on_mouse_down(
            MouseButton::Left,
            |e: &MouseDownEvent, window: &mut Window, _| {
                if e.click_count == 2 {
                    window.titlebar_double_click();
                }
            },
        )
        // Left / Start section: App Brand & Document Actions
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .when(is_rtl, |s| s.flex_row_reverse())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_1p5()
                        .py_1()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .child(
                            div()
                                .size(px(22.0))
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
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child("AnimaView"),
                        ),
                )
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
                                    "close-file-btn",
                                    crate::ui::icon::Icon::Cancel,
                                    theme,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.is_theme_dropdown_open = false;
                                        this.state.reset();
                                        cx.notify();
                                    }),
                                ),
                            )
                            .child(
                                render_header_btn(
                                    "export-btn",
                                    crate::ui::icon::Icon::Download,
                                    theme,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.is_theme_dropdown_open = false;
                                        this.state.active_modal = ActiveModal::Export;
                                        cx.notify();
                                    }),
                                ),
                            )
                            .child(
                                render_header_btn(
                                    "toggle-sidebar-btn",
                                    crate::ui::icon::Icon::SidebarRight,
                                    theme,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.is_theme_dropdown_open = false;
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
                            window.titlebar_double_click();
                        }
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
        // Right / End section: App Controls & Window Buttons
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .when(is_rtl, |s| s.flex_row_reverse())
                // File Open Button
                .child(
                    render_header_btn(
                        "open-file-action-btn",
                        crate::ui::icon::Icon::Folder,
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.state.is_theme_dropdown_open = false;
                            this.open_file_dialog(cx);
                        }),
                    ),
                )
                // Theme Dropdown
                .child(render_theme_dropdown(state, theme, is_rtl, cx))
                // Preferences Button
                .child(
                    render_header_btn(
                        "preferences-btn",
                        crate::ui::icon::Icon::Settings,
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.state.is_theme_dropdown_open = false;
                            this.state.active_modal = ActiveModal::Preferences;
                            cx.notify();
                        }),
                    ),
                )
                // About Button
                .child(
                    render_header_btn(
                        "about-btn",
                        crate::ui::icon::Icon::Info,
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.state.is_theme_dropdown_open = false;
                            this.state.active_modal = ActiveModal::About;
                            cx.notify();
                        }),
                    ),
                )
                // Divider before Window Controls
                .child(
                    div()
                        .w(px(1.0))
                        .h(px(16.0))
                        .mx_1()
                        .bg(theme.border),
                )
                // Window Control Buttons (Minimize, Maximize/Restore, Close)
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
                ),
        )
        .into_any_element()
}

fn render_theme_dropdown(
    state: &AppState,
    theme: &ThemeColors,
    is_rtl: bool,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let current_theme = state.config.theme;
    let is_open = state.is_theme_dropdown_open;

    let (icon, label) = match current_theme {
        Theme::System => (crate::ui::icon::Icon::Computer, t!("theme_system")),
        Theme::Light => (crate::ui::icon::Icon::Sun, t!("theme_light")),
        Theme::Dark => (crate::ui::icon::Icon::Moon, t!("theme_dark")),
    };

    div()
        .id("theme-dropdown-container")
        .relative()
        .child(
            div()
                .id("theme-dropdown-trigger")
                .h(px(28.0))
                .px_2()
                .gap_1p5()
                .rounded_md()
                .flex()
                .items_center()
                .justify_center()
                .when(is_rtl, |s| s.flex_row_reverse())
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if is_open { theme.text_primary } else { theme.text_secondary })
                .bg(if is_open { theme.surface_active } else { theme.surface })
                .border_1()
                .border_color(if is_open { theme.accent } else { theme.border })
                .hover(|s| s.bg(theme.surface_hover).text_color(theme.text_primary))
                .active(|s| s.bg(theme.surface_active))
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.state.is_theme_dropdown_open = !this.state.is_theme_dropdown_open;
                        cx.notify();
                    }),
                )
                .child(
                    crate::ui::icon::render_icon(icon)
                        .size(px(13.0))
                        .text_color(if is_open { theme.text_primary } else { theme.text_secondary }),
                )
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .child(label.to_string()),
                )
                .child(
                    crate::ui::icon::render_icon(crate::ui::icon::Icon::ArrowDown)
                        .size(px(11.0))
                        .text_color(theme.text_muted),
                ),
        )
        .children(if is_open {
            Some(
                deferred(
                    div()
                        .id("theme-dropdown-menu")
                        .absolute()
                        .top(px(32.0))
                        .when_else(is_rtl, |s| s.left_0(), |s| s.right_0())
                        .w(px(136.0))
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
                        .child(render_theme_option(
                            "theme-opt-system",
                            crate::ui::icon::Icon::Computer,
                            t!("theme_system").to_string(),
                            current_theme == Theme::System,
                            Theme::System,
                            theme,
                            is_rtl,
                            cx,
                        ))
                        .child(render_theme_option(
                            "theme-opt-light",
                            crate::ui::icon::Icon::Sun,
                            t!("theme_light").to_string(),
                            current_theme == Theme::Light,
                            Theme::Light,
                            theme,
                            is_rtl,
                            cx,
                        ))
                        .child(render_theme_option(
                            "theme-opt-dark",
                            crate::ui::icon::Icon::Moon,
                            t!("theme_dark").to_string(),
                            current_theme == Theme::Dark,
                            Theme::Dark,
                            theme,
                            is_rtl,
                            cx,
                        )),
                )
            )
        } else {
            None
        })
}

fn render_theme_option(
    id: &'static str,
    icon: crate::ui::icon::Icon,
    label: String,
    is_selected: bool,
    target_theme: Theme,
    theme: &ThemeColors,
    is_rtl: bool,
    cx: &mut Context<MainView>,
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
        .font_weight(if is_selected { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
        .text_color(if is_selected { theme.accent } else { theme.text_primary })
        .bg(if is_selected { theme.surface_active } else { theme.surface })
        .hover(|s| s.bg(theme.surface_hover))
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.state.update_theme(target_theme);
                cx.notify();
            }),
        )
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
                        .text_color(if is_selected { theme.accent } else { theme.text_primary }),
                )
                .child(
                    div()
                        .child(label),
                ),
        )
        .child(
            div()
                .w(px(14.0))
                .flex()
                .items_center()
                .justify_center()
                .children(if is_selected {
                    Some(
                        crate::ui::icon::render_icon(crate::ui::icon::Icon::Tick)
                            .size(px(14.0))
                            .text_color(theme.accent)
                    )
                } else {
                    None
                }),
        )
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
                .text_color(theme.text_secondary)
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
                .text_color(theme.text_secondary)
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
                .text_color(theme.text_secondary)
        )
}
