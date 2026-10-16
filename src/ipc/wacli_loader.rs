use std::process::Command;
use serde::Deserialize;
use crate::db::repository::{ChatRecord, MessageRecord};
use crate::ipc::protocol::{ConnectionState, MessageStatus};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub(crate) struct WacliResponse<T> {
    #[allow(dead_code)]
    pub(crate) success: bool,
    pub(crate) data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WacliChatDto {
    pub(crate) jid: String,
    pub(crate) name: Option<String>,
    pub(crate) pinned: Option<bool>,
    pub(crate) unread_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WacliMessagesData {
    pub(crate) messages: Option<Vec<WacliMessageDto>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct WacliMessageDto {
    pub(crate) chat_jid: String,
    pub(crate) sender_jid: Option<String>,
    pub(crate) msg_id: String,
    pub(crate) timestamp: Option<String>,
    pub(crate) from_me: bool,
    pub(crate) text: Option<String>,
    pub(crate) display_text: Option<String>,
}

pub struct WacliLoader;

impl WacliLoader {
    /// Loads chats and messages from local wacli installation into AppState
    pub fn load_initial_state() -> AppState {
        let db_dir = std::env::var("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("HOME")
                    .map(|h| std::path::PathBuf::from(h).join(".local/share/wazzup"))
                    .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/wazzup"))
            });
        let db_path = db_dir.join("wazzup.db");
        let store = crate::db::ChatStore::open(&db_path)
            .unwrap_or_else(|_| crate::db::ChatStore::new_in_memory().unwrap());

        let mut state = AppState::with_store(store);

        // 1. Fetch chats
        if let Ok(chats) = Self::fetch_chats() {
            for chat in &chats {
                let _ = state.store.upsert_chat(chat);
            }
            state.chats = chats;
        } else {
            // Offline fallback: restore cached chats from SQLite store
            state.chats = state.store.list_chats();
        }

        // 2. Fetch recent messages
        if let Ok(messages) = Self::fetch_recent_messages() {
            for msg in messages {
                let _ = state.store.insert_message(&msg);
                let list = state.messages_by_chat.entry(msg.chat_jid.clone()).or_default();
                list.push(msg);
            }
        }

        // 3. Mark as connected if we have chats
        if !state.chats.is_empty() {
            state.connection_state = ConnectionState::Connected;
            // Select first chat (usually pinned or most recent)
            if let Some(first_jid) = state.chats.first().map(|c| c.jid.clone()) {
                state.select_chat(Some(first_jid.clone()));
                if let Ok(msgs) = state.store.get_messages(&first_jid, 50) {
                    state.messages_by_chat.insert(first_jid, msgs);
                }
            }
        }

        state
    }

    pub fn fetch_chats() -> Result<Vec<ChatRecord>, String> {
        let output = Command::new("wacli")
            .args(["chats", "list", "--json", "--limit", "50"])
            .output()
            .map_err(|e| format!("Failed to execute wacli chats list: {}", e))?;

        if !output.status.success() {
            return Err("wacli returned non-zero exit code".to_string());
        }

        let resp: WacliResponse<Vec<WacliChatDto>> = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        let mut chats = Vec::new();
        if let Some(data) = resp.data {
            for dto in data {
                chats.push(ChatRecord {
                    jid: dto.jid,
                    name: dto.name,
                    unread_count: dto.unread_count.unwrap_or(0),
                    last_message_timestamp: None,
                    pinned: dto.pinned.unwrap_or(false),
                });
            }
        }

        Ok(chats)
    }

    pub fn fetch_recent_messages() -> Result<Vec<MessageRecord>, String> {
        let output = Command::new("wacli")
            .args(["messages", "list", "--json", "--limit", "100"])
            .output()
            .map_err(|e| format!("Failed to execute wacli messages list: {}", e))?;

        if !output.status.success() {
            return Err("wacli returned non-zero exit code".to_string());
        }

        let resp: WacliResponse<WacliMessagesData> = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        let mut messages = Vec::new();
        if let Some(data) = resp.data {
            if let Some(raw_msgs) = data.messages {
                for m in raw_msgs {
                    let body = m.text.or(m.display_text).unwrap_or_default();
                    let sender_jid = m.sender_jid.unwrap_or_else(|| m.chat_jid.clone());
                    let ts_sec = m.timestamp
                        .as_deref()
                        .and_then(parse_iso_seconds)
                        .unwrap_or(0);

                    messages.push(MessageRecord {
                        id: m.msg_id,
                        chat_jid: m.chat_jid,
                        sender_jid,
                        timestamp: ts_sec,
                        from_me: m.from_me,
                        body,
                        status: if m.from_me { MessageStatus::Read } else { MessageStatus::Delivered },
                        media: None,
                    });
                }
            }
        }

        // Sort chronologically ascending
        messages.sort_by_key(|m| m.timestamp);

        Ok(messages)
    }

    pub fn fetch_chat_messages(chat_jid: &str, limit: u32) -> Result<Vec<MessageRecord>, String> {
        let limit_str = limit.to_string();
        let output = Command::new("wacli")
            .args(["messages", "list", "--chat", chat_jid, "--json", "--limit", &limit_str])
            .output()
            .map_err(|e| format!("Failed to execute wacli messages list for chat: {}", e))?;

        if !output.status.success() {
            return Err("wacli returned non-zero exit code".to_string());
        }

        let resp: WacliResponse<WacliMessagesData> = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        let mut messages = Vec::new();
        if let Some(data) = resp.data {
            if let Some(raw_msgs) = data.messages {
                for m in raw_msgs {
                    let body = m.text.or(m.display_text).unwrap_or_default();
                    let sender_jid = m.sender_jid.unwrap_or_else(|| m.chat_jid.clone());
                    let ts_sec = m.timestamp
                        .as_deref()
                        .and_then(parse_iso_seconds)
                        .unwrap_or(0);

                    messages.push(MessageRecord {
                        id: m.msg_id,
                        chat_jid: m.chat_jid,
                        sender_jid,
                        timestamp: ts_sec,
                        from_me: m.from_me,
                        body,
                        status: if m.from_me { MessageStatus::Read } else { MessageStatus::Delivered },
                        media: None,
                    });
                }
            }
        }

        // Sort chronologically ascending
        messages.sort_by_key(|m| m.timestamp);

        Ok(messages)
    }
}

pub(crate) fn parse_iso_seconds(iso: &str) -> Option<i64> {
    // Basic parser for "YYYY-MM-DDTHH:MM:SSZ" without external chrono dependency
    if iso.len() >= 19 {
        let h: i64 = iso[11..13].parse().ok()?;
        let m: i64 = iso[14..16].parse().ok()?;
        let s: i64 = iso[17..19].parse().ok()?;
        Some(h * 3600 + m * 60 + s)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[core::prelude::v1::test]
    fn test_parse_iso_seconds() {
        assert_eq!(parse_iso_seconds("2026-10-16T14:32:59Z"), Some(14 * 3600 + 32 * 60 + 59));
        assert_eq!(parse_iso_seconds("invalid"), None);
    }

    #[core::prelude::v1::test]
    fn test_parse_wacli_chat_json() {
        let json = r#"{"success":true,"data":[{"jid":"123@s.whatsapp.net","kind":"dm","name":"Alice","last_message_ts":"2026-10-16T14:32:59Z","archived":false,"pinned":true,"muted_until":0,"unread":true,"unread_count":1}],"error":null}"#;
        let resp: WacliResponse<Vec<WacliChatDto>> = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        let chats = resp.data.unwrap();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].name.as_deref(), Some("Alice"));
        assert_eq!(chats[0].pinned, Some(true));
    }
}
