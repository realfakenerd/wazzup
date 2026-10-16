use gpui::{div, px, rgb, Context, IntoElement, ParentElement, Render, Styled, Window};
use crate::desktop::shortcuts::{DesktopSettings, VimMode};
use crate::ipc::protocol::ConnectionState;
use crate::state::AppState;
use crate::theme::ThemeColors;
use crate::ui::chat_list::render_chat_list;
use crate::ui::composer::{render_composer, ComposerState};
use crate::ui::message_thread::render_message_thread;

pub struct MainWindow {
    pub title: String,
    pub state: AppState,
    pub composer: ComposerState,
    pub desktop: DesktopSettings,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            title: "Wazzup".to_string(),
            state: AppState::new(),
            composer: ComposerState::new(),
            desktop: DesktopSettings::new(),
        }
    }

    pub fn with_state(state: AppState) -> Self {
        Self {
            title: "Wazzup".to_string(),
            state,
            composer: ComposerState::new(),
            desktop: DesktopSettings::new(),
        }
    }

    fn status_color(&self) -> gpui::Rgba {
        match self.state.connection_state {
            ConnectionState::Connected => ThemeColors::emerald(),
            ConnectionState::Connecting => rgb(0xf59e0b), // Amber
            ConnectionState::QrReady => rgb(0x3b82f6),    // Blue
            ConnectionState::Disconnected => rgb(ThemeColors::TEXT_MUTED),
        }
    }

    fn status_label(&self) -> String {
        match self.state.connection_state {
            ConnectionState::Connected => {
                if let Some(ref name) = self.state.push_name {
                    format!("Connected ({})", name)
                } else if let Some(ref phone) = self.state.phone {
                    format!("Connected ({})", phone)
                } else {
                    "Connected".to_string()
                }
            }
            ConnectionState::Connecting => "Connecting...".to_string(),
            ConnectionState::QrReady => "QR Code Ready".to_string(),
            ConnectionState::Disconnected => "Offline".to_string(),
        }
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let active_jid = self.state.active_chat_jid.as_deref();
        let filtered_chats = self.state.filtered_chats();
        let active_chat = self
            .state
            .chats
            .iter()
            .find(|c| Some(c.jid.as_str()) == active_jid);
        let active_messages = self.state.active_messages();
        let has_active_chat = active_chat.is_some();

        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(ThemeColors::base())
            .text_color(ThemeColors::text_primary())
            // Top App Bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h_10()
                    .px_4()
                    .bg(ThemeColors::mantle())
                    .border_b_1()
                    .border_color(rgb(ThemeColors::BORDER))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w_2p5()
                                    .h_2p5()
                                    .rounded_full()
                                    .bg(self.status_color()),
                            )
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(ThemeColors::text_primary())
                                    .child(self.title.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(ThemeColors::text_muted())
                                    .child(format!("• {}", self.status_label())),
                            )
                            .child(if self.desktop.privacy_mode {
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(rgb(0x7f1d1d))
                                    .text_size(px(10.0))
                                    .text_color(rgb(0xfecaca))
                                    .child("🔒 Privacy")
                            } else {
                                div()
                            })
                            .child(if self.desktop.vim_mode_enabled {
                                let label = match self.desktop.vim_mode {
                                    VimMode::Normal => "VIM: NORMAL",
                                    VimMode::Insert => "VIM: INSERT",
                                };
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(rgb(ThemeColors::SURFACE))
                                    .text_size(px(10.0))
                                    .text_color(ThemeColors::emerald())
                                    .child(label)
                            } else {
                                div()
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(ThemeColors::text_muted())
                            .child("Wayland • GPUI 0.2"),
                    ),
            )
            // Main Content Area: Sidebar + Chat Thread & Composer
            .child(
                div()
                    .flex_1()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    // Left Column: Chat List Sidebar
                    .child(render_chat_list(
                        &filtered_chats,
                        active_jid,
                        &self.state.search_query,
                        |jid| {
                            if self.desktop.privacy_mode {
                                Some("••••••••••••")
                            } else {
                                self.state
                                    .messages_by_chat
                                    .get(jid)
                                    .and_then(|msgs| msgs.last())
                                    .map(|m| m.body.as_str())
                            }
                        },
                    ))
                    // Right Column: Message Thread & Composer
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .h_full()
                            .child(render_message_thread(active_chat, active_messages))
                            .child(render_composer(&self.composer, has_active_chat)),
                    ),
            );

        // Search Palette Modal Overlay
        if self.desktop.search_palette_open {
            root = root.child(
                div()
                    .absolute()
                    .top_12()
                    .left_1_2()
                    .w(px(480.0))
                    .p_3()
                    .rounded_xl()
                    .bg(ThemeColors::mantle())
                    .border_1()
                    .border_color(ThemeColors::emerald())
                    .shadow_lg()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .text_color(ThemeColors::text_primary())
                                    .child("🔍 Search conversations:"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(ThemeColors::text_muted())
                                    .child("(Esc to close)"),
                            ),
                    ),
            );
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::ChatRecord;

    #[core::prelude::v1::test]
    fn test_main_window_creation() {
        let window = MainWindow::new();
        assert_eq!(window.title, "Wazzup");
        assert_eq!(window.state.connection_state, ConnectionState::Disconnected);
        assert!(!window.desktop.privacy_mode);
        assert!(!window.desktop.search_palette_open);
    }

    #[core::prelude::v1::test]
    fn test_main_window_with_active_chat() {
        let mut state = AppState::new();
        state.chats.push(ChatRecord {
            jid: "alice@s.whatsapp.net".into(),
            name: Some("Alice".into()),
            unread_count: 0,
            last_message_timestamp: Some(100),
            pinned: false,
        });
        state.select_chat(Some("alice@s.whatsapp.net".into()));

        let window = MainWindow::with_state(state);
        assert_eq!(
            window.state.active_chat_jid.as_deref(),
            Some("alice@s.whatsapp.net")
        );
    }

    #[core::prelude::v1::test]
    fn test_main_window_with_privacy_mode() {
        let mut window = MainWindow::new();
        window.desktop.toggle_privacy();
        assert!(window.desktop.privacy_mode);
    }

    #[core::prelude::v1::test]
    fn test_main_window_with_search_palette() {
        let mut window = MainWindow::new();
        window.desktop.toggle_search_palette();
        assert!(window.desktop.search_palette_open);
    }
}
