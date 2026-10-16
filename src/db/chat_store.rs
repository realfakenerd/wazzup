use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use rusqlite::{Connection, Result};

use crate::db::repository::{ChatRecord, ChatRepository, MessageRecord, MessageRepository};
use crate::db::schema::initialize_schema;

#[derive(Default)]
struct L1Cache {
    chats: Vec<ChatRecord>,
    messages_by_chat: HashMap<String, Vec<MessageRecord>>,
}

#[derive(Clone)]
pub struct ChatStore {
    conn: Arc<Mutex<Connection>>,
    cache: Arc<Mutex<L1Cache>>,
}

impl std::fmt::Debug for ChatStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatStore").finish_non_exhaustive()
    }
}

impl PartialEq for ChatStore {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.conn, &other.conn) && Arc::ptr_eq(&self.cache, &other.cache)
    }
}

impl Eq for ChatStore {}

impl ChatStore {
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn)?;
        Self::with_connection(conn)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        initialize_schema(&conn)?;
        let store = Self::with_connection(conn)?;
        store.init_from_db()?;
        Ok(store)
    }

    pub fn with_connection(conn: Connection) -> Result<Self> {
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            cache: Arc::new(Mutex::new(L1Cache::default())),
        })
    }

    pub fn init_from_db(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let chats = ChatRepository::list_all(&conn)?;
        let mut cache = self.cache.lock().unwrap();
        cache.chats = chats;
        Ok(())
    }

    pub fn upsert_chat(&self, chat: &ChatRecord) -> Result<()> {
        {
            let conn = self.conn.lock().unwrap();
            ChatRepository::upsert(&conn, chat)?;
        }

        let mut cache = self.cache.lock().unwrap();
        if let Some(existing) = cache.chats.iter_mut().find(|c| c.jid == chat.jid) {
            *existing = chat.clone();
        } else {
            cache.chats.push(chat.clone());
        }
        self.sort_chats(&mut cache.chats);
        Ok(())
    }

    pub fn list_chats(&self) -> Vec<ChatRecord> {
        let cache = self.cache.lock().unwrap();
        cache.chats.clone()
    }

    pub fn get_chat(&self, jid: &str) -> Option<ChatRecord> {
        let cache = self.cache.lock().unwrap();
        cache.chats.iter().find(|c| c.jid == jid).cloned()
    }

    pub fn clear_unread(&self, jid: &str) -> Result<()> {
        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE chats SET unread_count = 0 WHERE jid = ?1",
                rusqlite::params![jid],
            )?;
        }

        let mut cache = self.cache.lock().unwrap();
        if let Some(chat) = cache.chats.iter_mut().find(|c| c.jid == jid) {
            chat.unread_count = 0;
        }
        Ok(())
    }

    pub fn insert_message(&self, msg: &MessageRecord) -> Result<()> {
        {
            let conn = self.conn.lock().unwrap();
            // Ensure chat exists before inserting message due to foreign key
            if ChatRepository::get_by_jid(&conn, &msg.chat_jid)?.is_none() {
                ChatRepository::upsert(
                    &conn,
                    &ChatRecord {
                        jid: msg.chat_jid.clone(),
                        name: None,
                        unread_count: 0,
                        last_message_timestamp: Some(msg.timestamp),
                        pinned: false,
                    },
                )?;
            }

            MessageRepository::insert(&conn, msg)?;

            // Update chat's last message timestamp
            conn.execute(
                r#"
                UPDATE chats 
                SET last_message_timestamp = MAX(COALESCE(last_message_timestamp, 0), ?1)
                WHERE jid = ?2
                "#,
                rusqlite::params![msg.timestamp, msg.chat_jid],
            )?;
        }

        let mut cache = self.cache.lock().unwrap();
        // Update L1 message cache
        let list = cache
            .messages_by_chat
            .entry(msg.chat_jid.clone())
            .or_default();
        if !list.iter().any(|m| m.id == msg.id) {
            list.push(msg.clone());
            list.sort_by_key(|m| m.timestamp);
        }

        // Update L1 chat timestamp
        if let Some(chat) = cache.chats.iter_mut().find(|c| c.jid == msg.chat_jid) {
            chat.last_message_timestamp = Some(
                chat.last_message_timestamp
                    .map(|ts| ts.max(msg.timestamp))
                    .unwrap_or(msg.timestamp),
            );
        } else {
            cache.chats.push(ChatRecord {
                jid: msg.chat_jid.clone(),
                name: None,
                unread_count: 0,
                last_message_timestamp: Some(msg.timestamp),
                pinned: false,
            });
        }
        self.sort_chats(&mut cache.chats);

        Ok(())
    }

    pub fn get_messages(&self, chat_jid: &str, limit: u32) -> Result<Vec<MessageRecord>> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.messages_by_chat.get(chat_jid) {
                if !cached.is_empty() {
                    let start = cached.len().saturating_sub(limit as usize);
                    return Ok(cached[start..].to_vec());
                }
            }
        }

        // Cache miss: load from SQLite L2
        let msgs = {
            let conn = self.conn.lock().unwrap();
            MessageRepository::get_thread_messages(&conn, chat_jid, limit, 0)?
        };

        let mut cache = self.cache.lock().unwrap();
        cache
            .messages_by_chat
            .insert(chat_jid.to_string(), msgs.clone());

        Ok(msgs)
    }

    pub fn search_messages(&self, query: &str, limit: u32) -> Result<Vec<MessageRecord>> {
        let conn = self.conn.lock().unwrap();
        MessageRepository::search(&conn, query, limit)
    }

    fn sort_chats(&self, chats: &mut [ChatRecord]) {
        chats.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.last_message_timestamp.cmp(&a.last_message_timestamp))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::protocol::MessageStatus;

    #[test]
    fn test_chat_store_in_memory_crud() {
        let store = ChatStore::new_in_memory().expect("should create store");

        let chat = ChatRecord {
            jid: "alice@s.whatsapp.net".into(),
            name: Some("Alice".into()),
            unread_count: 2,
            last_message_timestamp: Some(100),
            pinned: true,
        };

        store.upsert_chat(&chat).expect("upsert should succeed");
        let chats = store.list_chats();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].jid, "alice@s.whatsapp.net");
        assert_eq!(chats[0].unread_count, 2);

        store.clear_unread("alice@s.whatsapp.net").unwrap();
        assert_eq!(store.get_chat("alice@s.whatsapp.net").unwrap().unread_count, 0);
    }

    #[test]
    fn test_chat_store_write_through_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        {
            let store = ChatStore::open(&db_path).unwrap();
            let msg = MessageRecord {
                id: "msg_1".into(),
                chat_jid: "bob@s.whatsapp.net".into(),
                sender_jid: "bob@s.whatsapp.net".into(),
                timestamp: 200,
                from_me: false,
                body: "Hello persistence".into(),
                status: MessageStatus::Read,
                media: None,
            };
            store.insert_message(&msg).unwrap();
        }

        // Re-open from disk (L2) and verify restore
        {
            let store = ChatStore::open(&db_path).unwrap();
            let chats = store.list_chats();
            assert_eq!(chats.len(), 1);
            assert_eq!(chats[0].jid, "bob@s.whatsapp.net");

            let msgs = store.get_messages("bob@s.whatsapp.net", 10).unwrap();
            assert_eq!(msgs.len(), 1);
            assert_eq!(msgs[0].body, "Hello persistence");
        }
    }

    #[test]
    fn test_chat_store_message_search() {
        let store = ChatStore::new_in_memory().unwrap();
        let msg1 = MessageRecord {
            id: "m1".into(),
            chat_jid: "c1@s.whatsapp.net".into(),
            sender_jid: "c1@s.whatsapp.net".into(),
            timestamp: 100,
            from_me: false,
            body: "Secret password inside".into(),
            status: MessageStatus::Read,
            media: None,
        };
        let msg2 = MessageRecord {
            id: "m2".into(),
            chat_jid: "c1@s.whatsapp.net".into(),
            sender_jid: "c1@s.whatsapp.net".into(),
            timestamp: 101,
            from_me: false,
            body: "Just normal chat".into(),
            status: MessageStatus::Read,
            media: None,
        };
        store.insert_message(&msg1).unwrap();
        store.insert_message(&msg2).unwrap();

        let results = store.search_messages("password", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "m1");
    }
}
