use gpui::{div, px, rgb, Context, Div, FocusHandle, InteractiveElement, MouseButton, ParentElement, Styled};
use crate::db::repository::ChatRecord;
use crate::theme::ThemeColors;
use crate::ui::chat_item::{render_chat_item, ChatItemProps};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatListAction {
    ChatSelected(String),
    SearchChanged(String),
    SearchCleared,
    None,
}

#[derive(Debug, Clone, Default)]
pub struct ChatListState {
    pub search_query: String,
    pub is_searching: bool,
    pub active_chat_jid: Option<String>,
}

impl ChatListState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_search_query(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.is_searching = !query.trim().is_empty();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.is_searching = false;
    }

    pub fn push_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.is_searching = true;
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
        if self.search_query.is_empty() {
            self.is_searching = false;
        }
    }

    pub fn filtered_chats<'a>(&self, chats: &'a [ChatRecord]) -> Vec<&'a ChatRecord> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            return chats.iter().collect();
        }

        chats
            .iter()
            .filter(|chat| {
                let name_match = chat
                    .name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&query))
                    .unwrap_or(false);
                let jid_match = chat.jid.to_lowercase().contains(&query);
                name_match || jid_match
            })
            .collect()
    }

    pub fn navigate(&mut self, chats: &[ChatRecord], delta: i32) -> Option<String> {
        let filtered = self.filtered_chats(chats);
        if filtered.is_empty() {
            return None;
        }

        let current_idx = self
            .active_chat_jid
            .as_deref()
            .and_then(|jid| filtered.iter().position(|c| c.jid == jid))
            .unwrap_or(0);

        let new_idx = if delta > 0 {
            (current_idx + delta as usize).min(filtered.len() - 1)
        } else {
            current_idx.saturating_sub((-delta) as usize)
        };

        let selected = filtered.get(new_idx).map(|c| c.jid.clone());
        if let Some(ref jid) = selected {
            self.active_chat_jid = Some(jid.clone());
        }
        selected
    }

    pub fn handle_key(&mut self, key: &str, ctrl: bool, alt: bool, chats: &[ChatRecord]) -> ChatListAction {
        if key == "Escape" || key == "esc" {
            if !self.search_query.is_empty() {
                self.clear_search();
                return ChatListAction::SearchCleared;
            }
            return ChatListAction::None;
        }

        if key == "Backspace" {
            if !self.search_query.is_empty() {
                self.pop_search_char();
                return ChatListAction::SearchChanged(self.search_query.clone());
            }
            return ChatListAction::None;
        }

        if key == "ArrowDown" || key == "down" {
            if let Some(jid) = self.navigate(chats, 1) {
                return ChatListAction::ChatSelected(jid);
            }
            return ChatListAction::None;
        }

        if key == "ArrowUp" || key == "up" {
            if let Some(jid) = self.navigate(chats, -1) {
                return ChatListAction::ChatSelected(jid);
            }
            return ChatListAction::None;
        }

        // Search text entry: single printable character without Ctrl/Alt
        if !ctrl && !alt && key.chars().count() == 1 {
            if let Some(c) = key.chars().next() {
                self.push_search_char(c);
                return ChatListAction::SearchChanged(self.search_query.clone());
            }
        }

        ChatListAction::None
    }
}

pub struct ChatListController {
    pub state: ChatListState,
    pub focus_handle: Option<FocusHandle>,
}

impl ChatListController {
    pub fn new() -> Self {
        Self {
            state: ChatListState::new(),
            focus_handle: None,
        }
    }

    pub fn with_focus_handle(focus_handle: FocusHandle) -> Self {
        Self {
            state: ChatListState::new(),
            focus_handle: Some(focus_handle),
        }
    }

    pub fn render_sidebar<'a, V: 'static>(
        &self,
        chats: &[&'a ChatRecord],
        active_jid: Option<&str>,
        privacy_mode: bool,
        last_snippets: impl Fn(&str) -> Option<&'a str>,
        cx: &mut Context<V>,
        on_select: impl Fn(&mut V, &str, &mut Context<V>) + 'static + Copy,
        on_clear_search: impl Fn(&mut V, &mut Context<V>) + 'static + Copy,
    ) -> Div {
        let search_query = &self.state.search_query;

        let mut sidebar = div()
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
                                    .text_color(if search_query.is_empty() {
                                        ThemeColors::text_muted()
                                    } else {
                                        ThemeColors::text_primary()
                                    })
                                    .child(if search_query.is_empty() {
                                        "Search chats (type or Ctrl+K)...".to_string()
                                    } else {
                                        search_query.clone()
                                    }),
                            )
                            .child(if !search_query.is_empty() {
                                div()
                                    .cursor_pointer()
                                    .text_size(px(12.0))
                                    .text_color(ThemeColors::text_muted())
                                    .child("✕")
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |view, _, _, cx| {
                                        on_clear_search(view, cx);
                                    }))
                            } else {
                                div()
                            }),
                    ),
            );

        if chats.is_empty() {
            sidebar = sidebar.child(
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
            let mut scroll = div().flex_1().flex().flex_col().overflow_hidden();

            for chat in chats {
                let chat_jid = chat.jid.clone();
                let is_selected = active_jid.map(|j| j == chat.jid).unwrap_or(false);
                let snippet = if privacy_mode {
                    Some("••••••••••••")
                } else {
                    last_snippets(&chat.jid)
                };
                let props = ChatItemProps::from_record(chat, snippet, is_selected);
                let item = render_chat_item(&props).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |view, _, _, cx| {
                        on_select(view, &chat_jid, cx);
                    }),
                );
                scroll = scroll.child(item);
            }

            sidebar = sidebar.child(scroll);
        }

        sidebar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chats() -> Vec<ChatRecord> {
        vec![
            ChatRecord {
                jid: "alice@s.whatsapp.net".into(),
                name: Some("Alice Johnson".into()),
                unread_count: 2,
                last_message_timestamp: Some(300),
                pinned: true,
            },
            ChatRecord {
                jid: "bob@s.whatsapp.net".into(),
                name: Some("Bob Smith".into()),
                unread_count: 0,
                last_message_timestamp: Some(200),
                pinned: false,
            },
            ChatRecord {
                jid: "charlie@s.whatsapp.net".into(),
                name: Some("Charlie Brown".into()),
                unread_count: 0,
                last_message_timestamp: Some(100),
                pinned: false,
            },
        ]
    }

    #[core::prelude::v1::test]
    fn test_chat_list_state_filter() {
        let chats = create_test_chats();
        let mut state = ChatListState::new();

        assert_eq!(state.filtered_chats(&chats).len(), 3);

        state.set_search_query("alice");
        let results = state.filtered_chats(&chats);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].jid, "alice@s.whatsapp.net");

        state.clear_search();
        assert_eq!(state.filtered_chats(&chats).len(), 3);
    }

    #[core::prelude::v1::test]
    fn test_chat_list_navigation_bounds() {
        let chats = create_test_chats();
        let mut state = ChatListState::new();

        // Navigate forward
        let sel = state.navigate(&chats, 1);
        assert_eq!(sel, Some("bob@s.whatsapp.net".into()));

        // Navigate forward again
        let sel = state.navigate(&chats, 1);
        assert_eq!(sel, Some("charlie@s.whatsapp.net".into()));

        // Navigate past end should clamp
        let sel = state.navigate(&chats, 1);
        assert_eq!(sel, Some("charlie@s.whatsapp.net".into()));

        // Navigate backward
        let sel = state.navigate(&chats, -1);
        assert_eq!(sel, Some("bob@s.whatsapp.net".into()));
    }

    #[core::prelude::v1::test]
    fn test_chat_list_handle_key_search_and_nav() {
        let chats = create_test_chats();
        let mut state = ChatListState::new();

        // Type 'a'
        let action = state.handle_key("a", false, false, &chats);
        assert_eq!(action, ChatListAction::SearchChanged("a".into()));
        assert_eq!(state.search_query, "a");

        // Backspace
        let action = state.handle_key("Backspace", false, false, &chats);
        assert_eq!(action, ChatListAction::SearchChanged("".into()));
        assert_eq!(state.search_query, "");

        // Arrow down
        let action = state.handle_key("ArrowDown", false, false, &chats);
        assert_eq!(action, ChatListAction::ChatSelected("bob@s.whatsapp.net".into()));
    }

    #[core::prelude::v1::test]
    fn test_chat_list_empty_chats_navigation() {
        let mut state = ChatListState::new();
        assert_eq!(state.navigate(&[], 1), None);
        assert_eq!(state.navigate(&[], -1), None);
    }

    #[core::prelude::v1::test]
    fn test_chat_list_escape_clears_search() {
        let chats = create_test_chats();
        let mut state = ChatListState::new();
        state.set_search_query("alice");

        let action = state.handle_key("Escape", false, false, &chats);
        assert_eq!(action, ChatListAction::SearchCleared);
        assert_eq!(state.search_query, "");
        assert!(!state.is_searching);

        let action_empty = state.handle_key("Escape", false, false, &chats);
        assert_eq!(action_empty, ChatListAction::None);
    }
}
