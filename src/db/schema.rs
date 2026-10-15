use rusqlite::Connection;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub fn initialize_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        );

        CREATE TABLE IF NOT EXISTS chats (
            jid TEXT PRIMARY KEY,
            name TEXT,
            unread_count INTEGER NOT NULL DEFAULT 0,
            last_message_timestamp INTEGER,
            pinned INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_chats_timestamp ON chats(last_message_timestamp DESC);

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            chat_jid TEXT NOT NULL,
            sender_jid TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            from_me INTEGER NOT NULL DEFAULT 0,
            body TEXT NOT NULL,
            status TEXT NOT NULL,
            media_type TEXT,
            media_path TEXT,
            media_mimetype TEXT,
            media_size INTEGER,
            FOREIGN KEY(chat_jid) REFERENCES chats(jid) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_messages_chat_ts ON messages(chat_jid, timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp DESC);

        CREATE TABLE IF NOT EXISTS contacts (
            jid TEXT PRIMARY KEY,
            display_name TEXT,
            phone TEXT,
            avatar_path TEXT
        );
        "#,
    )?;

    // Record schema version if not set
    conn.execute(
        "INSERT OR IGNORE INTO schema_version (version) VALUES (?1)",
        [CURRENT_SCHEMA_VERSION],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_schema_creates_tables() {
        let conn = Connection::open_in_memory().expect("in-memory db failed");
        initialize_schema(&conn).expect("schema initialization failed");

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert!(tables.contains(&"chats".to_string()));
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"contacts".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));

        // Check schema version recorded
        let version: u32 = conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |r| r.get(0))
            .expect("should read schema version");
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_initialize_schema_is_idempotent() {
        let conn = Connection::open_in_memory().expect("in-memory db failed");
        initialize_schema(&conn).expect("initial schema failed");
        initialize_schema(&conn).expect("second schema run should be idempotent");
    }
}
