use std::collections::HashMap;
use crate::db::chat_store::ChatStore;
use crate::db::repository::{ChatRecord, MessageRecord};
use crate::ipc::protocol::{ChatEvent, ConnectionState, MessageEvent, WacliEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub connection_state: ConnectionState,
    pub phone: Option<String>,
    pub push_name: Option<String>,
    pub qr_code: Option<String>,
    pub active_chat_jid: Option<String>,
    pub chats: Vec<ChatRecord>,
    pub messages_by_chat: HashMap<String, Vec<MessageRecord>>,
    pub search_query: String,
    pub is_searching: bool,
    pub store: ChatStore,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let store = ChatStore::new_in_memory().expect("in-memory store should initialize");
        Self::with_store(store)
    }

    pub fn with_store(store: ChatStore) -> Self {
        let chats = store.list_chats();
        Self {
            connection_state: ConnectionState::Disconnected,
            phone: None,
            push_name: None,
            qr_code: None,
            active_chat_jid: None,
            chats,
            messages_by_chat: HashMap::new(),
            search_query: String::new(),
            is_searching: false,
            store,
        }
    }

    pub fn select_chat(&mut self, jid: Option<String>) {
        if let Some(ref selected_jid) = jid {
            if let Some(chat) = self.chats.iter_mut().find(|c| c.jid == *selected_jid) {
                chat.unread_count = 0;
            }
            let _ = self.store.clear_unread(selected_jid);
        }
        self.active_chat_jid = jid;
    }

    pub fn set_search_query(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.is_searching = !query.trim().is_empty();
    }

    pub fn active_messages(&self) -> &[MessageRecord] {
        match &self.active_chat_jid {
            Some(jid) => self
                .messages_by_chat
                .get(jid)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
            None => &[],
        }
    }

    pub fn filtered_chats(&self) -> Vec<&ChatRecord> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            return self.chats.iter().collect();
        }

        self.chats
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

    pub fn handle_event(&mut self, event: &WacliEvent) {
        match event {
            WacliEvent::Status(status) => {
                self.connection_state = status.state.clone();
                self.phone = status.phone.clone();
                self.push_name = status.push_name.clone();
                self.qr_code = status.qr_code.clone();
            }
            WacliEvent::Chat(chat_event) => {
                self.apply_chat_event(chat_event);
            }
            WacliEvent::Message(msg_event) => {
                self.apply_message_event(msg_event);
            }
            WacliEvent::Error(_) => {
                // Keep existing state, error events logged by supervisor
            }
        }
    }

    fn apply_chat_event(&mut self, chat: &ChatEvent) {
        let chat_record = if let Some(existing) = self.chats.iter_mut().find(|c| c.jid == chat.jid) {
            if let Some(ref name) = chat.name {
                existing.name = Some(name.clone());
            }
            existing.unread_count = chat.unread_count;
            if let Some(ts) = chat.last_message_timestamp {
                existing.last_message_timestamp = Some(
                    existing
                        .last_message_timestamp
                        .map(|t| t.max(ts))
                        .unwrap_or(ts),
                );
            }
            existing.pinned = chat.pinned;
            existing.clone()
        } else {
            let record = ChatRecord {
                jid: chat.jid.clone(),
                name: chat.name.clone(),
                unread_count: chat.unread_count,
                last_message_timestamp: chat.last_message_timestamp,
                pinned: chat.pinned,
            };
            self.chats.push(record.clone());
            record
        };
        let _ = self.store.upsert_chat(&chat_record);
        self.sort_chats();
    }

    fn apply_message_event(&mut self, msg: &MessageEvent) {
        let record = MessageRecord {
            id: msg.id.clone(),
            chat_jid: msg.chat_jid.clone(),
            sender_jid: msg.sender_jid.clone(),
            timestamp: msg.timestamp,
            from_me: msg.from_me,
            body: msg.body.clone(),
            status: msg.status.clone(),
            media: msg.media.clone(),
        };

        let messages = self.messages_by_chat.entry(msg.chat_jid.clone()).or_default();
        if let Some(pos) = messages.iter().position(|m| m.id == record.id) {
            messages[pos] = record.clone();
        } else {
            messages.push(record.clone());
        }

        // Write-through to SQLite L2 store
        let _ = self.store.insert_message(&record);

        // Update chat unread & timestamp
        let is_active = self.active_chat_jid.as_deref() == Some(&msg.chat_jid);
        if let Some(chat) = self.chats.iter_mut().find(|c| c.jid == msg.chat_jid) {
            if !msg.from_me && !is_active {
                chat.unread_count += 1;
            }
            chat.last_message_timestamp = Some(
                chat.last_message_timestamp
                    .map(|t| t.max(msg.timestamp))
                    .unwrap_or(msg.timestamp),
            );
        } else {
            self.chats.push(ChatRecord {
                jid: msg.chat_jid.clone(),
                name: None,
                unread_count: if !msg.from_me && !is_active { 1 } else { 0 },
                last_message_timestamp: Some(msg.timestamp),
                pinned: false,
            });
        }
        self.sort_chats();
    }

    fn sort_chats(&mut self) {
        self.chats.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.last_message_timestamp.cmp(&a.last_message_timestamp))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::protocol::{MessageStatus, StatusEvent};

    #[test]
    fn test_initial_state() {
        let state = AppState::new();
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
        assert!(state.active_chat_jid.is_none());
        assert!(state.chats.is_empty());
        assert!(state.active_messages().is_empty());
    }

    #[test]
    fn test_status_event_updates_state() {
        let mut state = AppState::new();
        let event = WacliEvent::Status(StatusEvent {
            state: ConnectionState::Connected,
            phone: Some("+123456789".to_string()),
            push_name: Some("Luke".to_string()),
            qr_code: None,
        });

        state.handle_event(&event);
        assert_eq!(state.connection_state, ConnectionState::Connected);
        assert_eq!(state.phone.as_deref(), Some("+123456789"));
        assert_eq!(state.push_name.as_deref(), Some("Luke"));
    }

    #[test]
    fn test_chat_selection_clears_unread() {
        let mut state = AppState::new();
        state.chats.push(ChatRecord {
            jid: "123@s.whatsapp.net".to_string(),
            name: Some("Alice".to_string()),
            unread_count: 5,
            last_message_timestamp: Some(100),
            pinned: false,
        });

        state.select_chat(Some("123@s.whatsapp.net".to_string()));
        assert_eq!(state.active_chat_jid.as_deref(), Some("123@s.whatsapp.net"));
        assert_eq!(state.chats[0].unread_count, 0);
    }

    #[test]
    fn test_message_event_appends_and_updates_chat() {
        let mut state = AppState::new();
        let msg = MessageEvent {
            id: "msg_1".to_string(),
            chat_jid: "123@s.whatsapp.net".to_string(),
            sender_jid: "123@s.whatsapp.net".to_string(),
            timestamp: 1500,
            from_me: false,
            body: "Hello world".to_string(),
            status: MessageStatus::Delivered,
            media: None,
        };

        state.handle_event(&WacliEvent::Message(msg));
        assert_eq!(state.chats.len(), 1);
        assert_eq!(state.chats[0].jid, "123@s.whatsapp.net");
        assert_eq!(state.chats[0].unread_count, 1);
        assert_eq!(state.chats[0].last_message_timestamp, Some(1500));

        state.select_chat(Some("123@s.whatsapp.net".to_string()));
        let active = state.active_messages();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].body, "Hello world");
    }

    #[test]
    fn test_search_filtering() {
        let mut state = AppState::new();
        state.chats.push(ChatRecord {
            jid: "alice@s.whatsapp.net".to_string(),
            name: Some("Alice Johnson".to_string()),
            unread_count: 0,
            last_message_timestamp: Some(100),
            pinned: false,
        });
        state.chats.push(ChatRecord {
            jid: "bob@s.whatsapp.net".to_string(),
            name: Some("Bob Smith".to_string()),
            unread_count: 0,
            last_message_timestamp: Some(200),
            pinned: false,
        });

        state.set_search_query("alice");
        let results = state.filtered_chats();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].jid, "alice@s.whatsapp.net");

        state.set_search_query("smith");
        let results = state.filtered_chats();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].jid, "bob@s.whatsapp.net");

        state.set_search_query("");
        let results = state.filtered_chats();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_app_state_with_store_offline_restore() {
        let store = ChatStore::new_in_memory().unwrap();
        let chat = ChatRecord {
            jid: "charlie@s.whatsapp.net".to_string(),
            name: Some("Charlie".to_string()),
            unread_count: 3,
            last_message_timestamp: Some(500),
            pinned: true,
        };
        store.upsert_chat(&chat).unwrap();

        let mut state = AppState::with_store(store);
        assert_eq!(state.chats.len(), 1);
        assert_eq!(state.chats[0].jid, "charlie@s.whatsapp.net");
        assert_eq!(state.chats[0].unread_count, 3);

        // Selecting chat clears unread count in both memory and store
        state.select_chat(Some("charlie@s.whatsapp.net".to_string()));
        assert_eq!(state.chats[0].unread_count, 0);
        assert_eq!(state.store.get_chat("charlie@s.whatsapp.net").unwrap().unread_count, 0);
    }
}
