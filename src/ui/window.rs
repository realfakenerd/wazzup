use std::sync::Arc;
use gpui::{
    div, px, rgb, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, Render, Styled, Window,
};
use crate::desktop::shortcuts::{DesktopSettings, ShortcutAction, VimMode};
use crate::ipc::protocol::ConnectionState;
use crate::ipc::client::WhatsAppClient;
use crate::state::AppState;
use crate::theme::ThemeColors;
use crate::ui::chat_list_controller::{ChatListAction, ChatListController};
use crate::ui::composer::{render_composer, ComposerState};
use crate::ui::message_thread::render_message_thread;

pub struct MainWindow {
    pub title: String,
    pub state: AppState,
    pub composer: ComposerState,
    pub desktop: DesktopSettings,
    pub client: Option<Arc<dyn WhatsAppClient>>,
    pub chat_list: ChatListController,
    pub focus_handle: Option<FocusHandle>,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            title: "Wazzup".to_string(),
            state: AppState::new(),
            composer: ComposerState::new(),
            desktop: DesktopSettings::new(),
            client: None,
            chat_list: ChatListController::new(),
            focus_handle: None,
        }
    }

    pub fn with_state(state: AppState) -> Self {
        Self {
            title: "Wazzup".to_string(),
            state,
            composer: ComposerState::new(),
            desktop: DesktopSettings::new(),
            client: None,
            chat_list: ChatListController::new(),
            focus_handle: None,
        }
    }

    pub fn with_client(state: AppState, client: Arc<dyn WhatsAppClient>) -> Self {
        Self {
            title: "Wazzup".to_string(),
            state,
            composer: ComposerState::new(),
            desktop: DesktopSettings::new(),
            client: Some(client),
            chat_list: ChatListController::new(),
            focus_handle: None,
        }
    }

    pub fn select_chat(&mut self, jid: &str) {
        self.state.select_chat(Some(jid.to_string()));
        self.chat_list.state.active_chat_jid = Some(jid.to_string());

        // Instant local render from ChatStore if not already loaded in memory
        if !self.state.messages_by_chat.contains_key(jid) {
            if let Ok(msgs) = self.state.store.get_messages(jid, 1000) {
                if !msgs.is_empty() {
                    self.state.messages_by_chat.insert(jid.to_string(), msgs);
                }
            }
        }
    }

    pub fn select_chat_async(&mut self, jid: &str, cx: &mut Context<Self>) {
        self.select_chat(jid);

        if let Some(client) = self.client.clone() {
            let jid_string = jid.to_string();
            cx.spawn(async move |this, cx| {
                if let Ok(msgs) = client.fetch_messages(&jid_string, 1000).await {
                    let _ = this.update(cx, |window, cx| {
                        for msg in &msgs {
                            let _ = window.state.store.insert_message(msg);
                        }
                        window.state.messages_by_chat.insert(jid_string, msgs);
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }

    pub fn navigate_chat(&mut self, delta: i32) {
        if let Some(jid) = self.chat_list.state.navigate(&self.state.chats, delta) {
            self.select_chat(&jid);
        }
    }

    pub fn navigate_chat_async(&mut self, delta: i32, cx: &mut Context<Self>) {
        if let Some(jid) = self.chat_list.state.navigate(&self.state.chats, delta) {
            self.select_chat_async(&jid, cx);
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
                    self.chat_list.state.is_searching = true;
                }
                ShortcutAction::Escape => {
                    self.state.search_query.clear();
                    self.state.is_searching = false;
                    self.chat_list.state.clear_search();
                }
                ShortcutAction::VimNextChat => {
                    self.navigate_chat_async(1, cx);
                }
                ShortcutAction::VimPrevChat => {
                    self.navigate_chat_async(-1, cx);
                }
                _ => {}
            }
            cx.notify();
            return;
        }

        // Delegate chat list key interactions (arrows, escape, backspace, search typing)
        match self.chat_list.state.handle_key(key, ctrl, alt, &self.state.chats) {
            ChatListAction::ChatSelected(jid) => {
                self.select_chat_async(&jid, cx);
                cx.notify();
            }
            ChatListAction::SearchChanged(query) => {
                self.state.set_search_query(&query);
                cx.notify();
            }
            ChatListAction::SearchCleared => {
                self.state.search_query.clear();
                self.state.is_searching = false;
                cx.notify();
            }
            ChatListAction::None => {}
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.get_or_insert_with(|| cx.focus_handle()).clone();
        if !focus_handle.is_focused(window) && !focus_handle.contains_focused(window, cx) {
            focus_handle.focus(window);
        }

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
            .track_focus(&focus_handle)
            .key_context("MainWindow")
            .flex()
            .flex_col()
            .size_full()
            .bg(ThemeColors::base())
            .text_color(ThemeColors::text_primary())
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, _| {
                if let Some(ref fh) = this.focus_handle {
                    fh.focus(window);
                }
            }))
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
                    // Left Column: Interactive Chat List Sidebar (Delegated to ChatListController)
                    .child(self.chat_list.render_sidebar(
                        &filtered_chats,
                        active_jid,
                        self.desktop.privacy_mode,
                        |jid| {
                            self.state
                                .messages_by_chat
                                .get(jid)
                                .and_then(|msgs| msgs.last())
                                .map(|m| m.body.as_str())
                        },
                        cx,
                        |this, jid, cx| {
                            this.chat_list.state.active_chat_jid = Some(jid.to_string());
                            this.select_chat_async(jid, cx);
                            cx.notify();
                        },
                        |this, cx| {
                            this.chat_list.state.clear_search();
                            this.state.search_query.clear();
                            this.state.is_searching = false;
                            cx.notify();
                        },
                    ))
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

    #[core::prelude::v1::test]
    fn test_main_window_with_client_and_select_chat() {
        let mock_client = std::sync::Arc::new(crate::ipc::client::MockWhatsAppClient::new());
        let mut state = AppState::new();
        state.chats.push(ChatRecord {
            jid: "chat1@s.whatsapp.net".into(),
            name: Some("Chat 1".into()),
            unread_count: 0,
            last_message_timestamp: Some(200),
            pinned: false,
        });

        let mut window = MainWindow::with_client(state, mock_client);
        assert!(window.client.is_some());

        // Calling select_chat is instant, never blocks
        window.select_chat("chat1@s.whatsapp.net");
        assert_eq!(window.state.active_chat_jid.as_deref(), Some("chat1@s.whatsapp.net"));
    }
}
