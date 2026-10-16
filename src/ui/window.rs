use gpui::{div, px, rgb, Context, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, Render, Styled, Window};
use crate::desktop::shortcuts::{DesktopSettings, ShortcutAction, VimMode};
use crate::ipc::protocol::ConnectionState;
use crate::ipc::WacliLoader;
use crate::state::AppState;
use crate::theme::ThemeColors;
use crate::ui::chat_item::{render_chat_item, ChatItemProps};
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

    pub fn select_chat(&mut self, jid: &str) {
        self.state.select_chat(Some(jid.to_string()));
        if self
            .state
            .messages_by_chat
            .get(jid)
            .map(|m| m.is_empty())
            .unwrap_or(true)
        {
            if let Ok(msgs) = WacliLoader::fetch_chat_messages(jid, 50) {
                self.state.messages_by_chat.insert(jid.to_string(), msgs);
            }
        }
    }

    pub fn navigate_chat(&mut self, delta: i32) {
        let filtered = self.state.filtered_chats();
        if filtered.is_empty() {
            return;
        }

        let current_idx = self
            .state
            .active_chat_jid
            .as_deref()
            .and_then(|jid| filtered.iter().position(|c| c.jid == jid))
            .unwrap_or(0);

        let new_idx = if delta > 0 {
            (current_idx + delta as usize).min(filtered.len() - 1)
        } else {
            current_idx.saturating_sub((-delta) as usize)
        };

        if let Some(target_chat) = filtered.get(new_idx) {
            let jid = target_chat.jid.clone();
            self.select_chat(&jid);
        }
    }

    pub fn handle_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = &event.keystroke.key;
        let ctrl = event.keystroke.modifiers.control;
        let shift = event.keystroke.modifiers.shift;
        let alt = event.keystroke.modifiers.alt;

        // Shortcuts
        if let Some(action) = self.desktop.handle_keypress(key, ctrl, shift) {
            match action {
                ShortcutAction::QuickSearch => {
                    self.state.is_searching = true;
                }
                ShortcutAction::Escape => {
                    self.state.search_query.clear();
                    self.state.is_searching = false;
                }
                ShortcutAction::VimNextChat => {
                    self.navigate_chat(1);
                }
                ShortcutAction::VimPrevChat => {
                    self.navigate_chat(-1);
                }
                _ => {}
            }
            cx.notify();
            return;
        }

        if key == "Escape" || key == "esc" {
            self.state.search_query.clear();
            self.state.is_searching = false;
            cx.notify();
            return;
        }

        if key == "Backspace" {
            if !self.state.search_query.is_empty() {
                self.state.search_query.pop();
                cx.notify();
            }
            return;
        }

        if key == "ArrowDown" || key == "down" {
            self.navigate_chat(1);
            cx.notify();
            return;
        }

        if key == "ArrowUp" || key == "up" {
            self.navigate_chat(-1);
            cx.notify();
            return;
        }

        // Search text entry: single printable character without Ctrl/Alt
        if !ctrl && !alt && key.chars().count() == 1 {
            self.state.search_query.push_str(key);
            self.state.is_searching = true;
            cx.notify();
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_jid = self.state.active_chat_jid.as_deref();
        let filtered_chats = self.state.filtered_chats();
        let active_chat = self
            .state
            .chats
            .iter()
            .find(|c| Some(c.jid.as_str()) == active_jid)
            .cloned();
        let active_messages = self.state.active_messages().to_vec();
        let has_active_chat = active_chat.is_some();

        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(ThemeColors::base())
            .text_color(ThemeColors::text_primary())
            .on_key_down(cx.listener(|this, event, _, cx| {
                this.handle_key(event, cx);
            }))
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
                    // Left Column: Interactive Chat List Sidebar
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_80()
                            .h_full()
                            .bg(ThemeColors::base())
                            .border_r_1()
                            .border_color(rgb(ThemeColors::BORDER))
                            // Search Header
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .h_12()
                                    .px_3()
                                    .bg(ThemeColors::mantle())
                                    .border_b_1()
                                    .border_color(rgb(ThemeColors::BORDER))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .w_full()
                                            .h_8()
                                            .px_3()
                                            .rounded_md()
                                            .bg(rgb(ThemeColors::SURFACE0))
                                            .cursor_pointer()
                                            .child(
                                                div()
                                                    .text_size(px(13.0))
                                                    .text_color(if self.state.search_query.is_empty() {
                                                        ThemeColors::text_muted()
                                                    } else {
                                                        ThemeColors::text_primary()
                                                    })
                                                    .child(if self.state.search_query.is_empty() {
                                                        "Search chats (type or Ctrl+K)...".to_string()
                                                    } else {
                                                        self.state.search_query.clone()
                                                    }),
                                            )
                                            .child(if !self.state.search_query.is_empty() {
                                                div()
                                                    .cursor_pointer()
                                                    .text_size(px(12.0))
                                                    .text_color(ThemeColors::text_muted())
                                                    .child("✕")
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.state.search_query.clear();
                                                        this.state.is_searching = false;
                                                        cx.notify();
                                                    }))
                                            } else {
                                                div()
                                            }),
                                    ),
                            )
                            // Scroll Area with clickable Chat items
                            .child({
                                let mut scroll = div().flex_1().flex().flex_col().overflow_hidden();
                                if filtered_chats.is_empty() {
                                    scroll = scroll.child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_size(px(13.0))
                                            .text_color(ThemeColors::text_muted())
                                            .child("No matching chats"),
                                    );
                                } else {
                                    for chat in &filtered_chats {
                                        let chat_jid = chat.jid.clone();
                                        let is_selected = active_jid.map(|j| j == chat.jid).unwrap_or(false);
                                        let snippet = if self.desktop.privacy_mode {
                                            Some("••••••••••••")
                                        } else {
                                            self.state
                                                .messages_by_chat
                                                .get(&chat.jid)
                                                .and_then(|msgs| msgs.last())
                                                .map(|m| m.body.as_str())
                                        };
                                        let props = ChatItemProps::from_record(chat, snippet, is_selected);
                                        let item = render_chat_item(&props)
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                this.select_chat(&chat_jid);
                                                cx.notify();
                                            }));
                                        scroll = scroll.child(item);
                                    }
                                }
                                scroll
                            }),
                    )
                    // Right Column: Message Thread & Composer
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .h_full()
                            .child(render_message_thread(active_chat.as_ref(), &active_messages))
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

    #[core::prelude::v1::test]
    fn test_main_window_navigate_chat() {
        let mut state = AppState::new();
        state.chats.push(ChatRecord {
            jid: "chat1@s.whatsapp.net".into(),
            name: Some("Chat 1".into()),
            unread_count: 0,
            last_message_timestamp: Some(200),
            pinned: false,
        });
        state.chats.push(ChatRecord {
            jid: "chat2@s.whatsapp.net".into(),
            name: Some("Chat 2".into()),
            unread_count: 0,
            last_message_timestamp: Some(100),
            pinned: false,
        });
        state.select_chat(Some("chat1@s.whatsapp.net".into()));

        let mut window = MainWindow::with_state(state);
        assert_eq!(window.state.active_chat_jid.as_deref(), Some("chat1@s.whatsapp.net"));

        // Navigate forward
        window.navigate_chat(1);
        assert_eq!(window.state.active_chat_jid.as_deref(), Some("chat2@s.whatsapp.net"));

        // Navigate backward
        window.navigate_chat(-1);
        assert_eq!(window.state.active_chat_jid.as_deref(), Some("chat1@s.whatsapp.net"));
    }
}
