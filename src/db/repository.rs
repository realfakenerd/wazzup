use rusqlite::{params, Connection, Result};
use crate::ipc::protocol::{MediaPayload, MediaType, MessageStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRecord {
    pub jid: String,
    pub name: Option<String>,
    pub unread_count: u32,
    pub last_message_timestamp: Option<i64>,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRecord {
    pub id: String,
    pub chat_jid: String,
    pub sender_jid: String,
    pub timestamp: i64,
    pub from_me: bool,
    pub body: String,
    pub status: MessageStatus,
    pub media: Option<MediaPayload>,
}

pub struct ChatRepository;

impl ChatRepository {
    pub fn upsert(conn: &Connection, chat: &ChatRecord) -> Result<()> {
        conn.execute(
            r#"
            INSERT INTO chats (jid, name, unread_count, last_message_timestamp, pinned)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(jid) DO UPDATE SET
                name = COALESCE(excluded.name, chats.name),
                unread_count = excluded.unread_count,
                last_message_timestamp = MAX(COALESCE(chats.last_message_timestamp, 0), COALESCE(excluded.last_message_timestamp, 0)),
                pinned = excluded.pinned
            "#,
            params![
                chat.jid,
                chat.name,
                chat.unread_count,
                chat.last_message_timestamp,
                chat.pinned as i32
            ],
        )?;
        Ok(())
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<ChatRecord>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT jid, name, unread_count, last_message_timestamp, pinned
            FROM chats
            ORDER BY pinned DESC, last_message_timestamp DESC NULLS LAST
            "#,
        )?;

        let chats = stmt
            .query_map([], |row| {
                Ok(ChatRecord {
                    jid: row.get(0)?,
                    name: row.get(1)?,
                    unread_count: row.get(2)?,
                    last_message_timestamp: row.get(3)?,
                    pinned: row.get::<_, i32>(4)? != 0,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(chats)
    }

    pub fn get_by_jid(conn: &Connection, jid: &str) -> Result<Option<ChatRecord>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT jid, name, unread_count, last_message_timestamp, pinned
            FROM chats
            WHERE jid = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![jid])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ChatRecord {
                jid: row.get(0)?,
                name: row.get(1)?,
                unread_count: row.get(2)?,
                last_message_timestamp: row.get(3)?,
                pinned: row.get::<_, i32>(4)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn mark_as_read(conn: &Connection, jid: &str) -> Result<()> {
        conn.execute(
            "UPDATE chats SET unread_count = 0 WHERE jid = ?1",
            params![jid],
        )?;
        Ok(())
    }
}

pub struct MessageRepository;

impl MessageRepository {
    pub fn insert(conn: &Connection, msg: &MessageRecord) -> Result<()> {
        let (m_type, m_path, m_mimetype, m_size) = match &msg.media {
            Some(m) => {
                let t = serde_json::to_string(&m.media_type).unwrap_or_default();
                (
                    Some(t.trim_matches('"').to_string()),
                    m.file_path.clone(),
                    Some(m.mimetype.clone()),
                    m.file_size,
                )
            }
            None => (None, None, None, None),
        };

        let status_str = serde_json::to_string(&msg.status).unwrap_or_default();
        let clean_status = status_str.trim_matches('"');

        // Ensure parent chat exists
        conn.execute(
            r#"
            INSERT OR IGNORE INTO chats (jid, last_message_timestamp)
            VALUES (?1, ?2)
            "#,
            params![msg.chat_jid, msg.timestamp],
        )?;

        conn.execute(
            r#"
            INSERT INTO messages (
                id, chat_jid, sender_jid, timestamp, from_me, body, status,
                media_type, media_path, media_mimetype, media_size
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                media_path = COALESCE(excluded.media_path, messages.media_path)
            "#,
            params![
                msg.id,
                msg.chat_jid,
                msg.sender_jid,
                msg.timestamp,
                msg.from_me as i32,
                msg.body,
                clean_status,
                m_type,
                m_path,
                m_mimetype,
                m_size
            ],
        )?;

        // Update chat last message timestamp
        conn.execute(
            r#"
            UPDATE chats
            SET last_message_timestamp = MAX(COALESCE(last_message_timestamp, 0), ?1)
            WHERE jid = ?2
            "#,
            params![msg.timestamp, msg.chat_jid],
        )?;

        Ok(())
    }

    pub fn update_status(conn: &Connection, id: &str, status: MessageStatus) -> Result<bool> {
        let status_str = serde_json::to_string(&status).unwrap_or_default();
        let clean_status = status_str.trim_matches('"');

        let rows = conn.execute(
            "UPDATE messages SET status = ?1 WHERE id = ?2",
            params![clean_status, id],
        )?;

        Ok(rows > 0)
    }

    pub fn get_thread_messages(
        conn: &Connection,
        chat_jid: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<MessageRecord>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, chat_jid, sender_jid, timestamp, from_me, body, status,
                   media_type, media_path, media_mimetype, media_size
            FROM messages
            WHERE chat_jid = ?1
            ORDER BY timestamp DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )?;

        let mut messages = stmt
            .query_map(params![chat_jid, limit, offset], |row| {
                let status_str: String = row.get(6)?;
                let status: MessageStatus = serde_json::from_str(&format!("\"{}\"", status_str))
                    .unwrap_or(MessageStatus::Sent);

                let media_type_str: Option<String> = row.get(7)?;
                let media = media_type_str.and_then(|mt| {
                    let media_type: Option<MediaType> =
                        serde_json::from_str(&format!("\"{}\"", mt)).ok();
                    media_type.map(|t| MediaPayload {
                        media_type: t,
                        mimetype: row.get(9).unwrap_or_default(),
                        file_path: row.get(8).unwrap_or(None),
                        file_name: None,
                        file_size: row.get(10).unwrap_or(None),
                    })
                });

                Ok(MessageRecord {
                    id: row.get(0)?,
                    chat_jid: row.get(1)?,
                    sender_jid: row.get(2)?,
                    timestamp: row.get(3)?,
                    from_me: row.get::<_, i32>(4)? != 0,
                    body: row.get(5)?,
                    status,
                    media,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        // Reverse to display chronologically ascending
        messages.reverse();
        Ok(messages)
    }

    pub fn search(conn: &Connection, query_str: &str, limit: u32) -> Result<Vec<MessageRecord>> {
        let pattern = format!("%{}%", query_str);
        let mut stmt = conn.prepare(
            r#"
            SELECT id, chat_jid, sender_jid, timestamp, from_me, body, status,
                   media_type, media_path, media_mimetype, media_size
            FROM messages
            WHERE body LIKE ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#,
        )?;

        let messages = stmt
            .query_map(params![pattern, limit], |row| {
                let status_str: String = row.get(6)?;
                let status: MessageStatus = serde_json::from_str(&format!("\"{}\"", status_str))
                    .unwrap_or(MessageStatus::Sent);

                Ok(MessageRecord {
                    id: row.get(0)?,
                    chat_jid: row.get(1)?,
                    sender_jid: row.get(2)?,
                    timestamp: row.get(3)?,
                    from_me: row.get::<_, i32>(4)? != 0,
                    body: row.get(5)?,
                    status,
                    media: None,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::initialize_schema;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("failed open memory db");
        initialize_schema(&conn).expect("failed init schema");
        conn
    }

    #[test]
    fn test_chat_crud() {
        let conn = setup_test_db();
        let chat = ChatRecord {
            jid: "123@s.whatsapp.net".to_string(),
            name: Some("Alice".to_string()),
            unread_count: 2,
            last_message_timestamp: Some(1000),
            pinned: true,
        };

        ChatRepository::upsert(&conn, &chat).expect("upsert failed");

        let fetched = ChatRepository::get_by_jid(&conn, "123@s.whatsapp.net")
            .expect("fetch failed")
            .expect("chat should exist");
        assert_eq!(fetched.name.as_deref(), Some("Alice"));
        assert_eq!(fetched.unread_count, 2);
        assert!(fetched.pinned);

        ChatRepository::mark_as_read(&conn, "123@s.whatsapp.net").expect("mark read failed");
        let updated = ChatRepository::get_by_jid(&conn, "123@s.whatsapp.net")
            .unwrap()
            .unwrap();
        assert_eq!(updated.unread_count, 0);
    }

    #[test]
    fn test_message_insert_and_pagination() {
        let conn = setup_test_db();
        let chat_jid = "chat1@s.whatsapp.net";

        for i in 1..=5 {
            let msg = MessageRecord {
                id: format!("msg_{}", i),
                chat_jid: chat_jid.to_string(),
                sender_jid: "sender@s.whatsapp.net".to_string(),
                timestamp: 1000 + i,
                from_me: i % 2 == 0,
                body: format!("Message number {}", i),
                status: MessageStatus::Sent,
                media: None,
            };
            MessageRepository::insert(&conn, &msg).expect("insert failed");
        }

        // Fetch thread messages limit 3, offset 0 (most recent 3 chronologically)
        let page1 = MessageRepository::get_thread_messages(&conn, chat_jid, 3, 0).unwrap();
        assert_eq!(page1.len(), 3);
        assert_eq!(page1[0].id, "msg_3");
        assert_eq!(page1[1].id, "msg_4");
        assert_eq!(page1[2].id, "msg_5");

        // Status update
        let updated = MessageRepository::update_status(&conn, "msg_5", MessageStatus::Read).unwrap();
        assert!(updated);
        let updated_msgs = MessageRepository::get_thread_messages(&conn, chat_jid, 1, 0).unwrap();
        assert_eq!(updated_msgs[0].status, MessageStatus::Read);
    }

    #[test]
    fn test_message_search() {
        let conn = setup_test_db();
        let chat_jid = "chat1@s.whatsapp.net";

        let msg1 = MessageRecord {
            id: "msg_a".to_string(),
            chat_jid: chat_jid.to_string(),
            sender_jid: "alice@s.whatsapp.net".to_string(),
            timestamp: 2000,
            from_me: false,
            body: "Need to buy groceries today".to_string(),
            status: MessageStatus::Read,
            media: None,
        };
        let msg2 = MessageRecord {
            id: "msg_b".to_string(),
            chat_jid: chat_jid.to_string(),
            sender_jid: "bob@s.whatsapp.net".to_string(),
            timestamp: 2001,
            from_me: true,
            body: "Meeting scheduled for 3pm".to_string(),
            status: MessageStatus::Delivered,
            media: None,
        };

        MessageRepository::insert(&conn, &msg1).unwrap();
        MessageRepository::insert(&conn, &msg2).unwrap();

        let search_results = MessageRepository::search(&conn, "groceries", 10).unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].id, "msg_a");
    }
}
