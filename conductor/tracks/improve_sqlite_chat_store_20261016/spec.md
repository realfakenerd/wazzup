# Track Specification: Consolidate Reactive SQLite ChatStore

**Track ID:** `improve_sqlite_chat_store_20261016`  
**Type:** `refactor`  
**Recommendation Strength:** `Strong` (`in-process`)  

---

## 1. Overview & Context

`AppState` currently stores chats and messages in volatile in-memory collections (`HashMap<String, Vec<MessageRecord>>`, `Vec<ChatRecord>`). Concurrently, `src/db/repository.rs` contains fully functional, tested `ChatRepository` and `MessageRepository` modules that are completely disconnected at runtime.

This track deepens persistence by establishing a single, unified `ChatStore` module that serves as the single source of truth, encapsulating an in-memory hot cache backed by write-through SQLite storage.

---

## 2. Before & After Architecture

### Before (Fragmented & Volatile)
```text
MainWindow
    │
    ▼
AppState (Volatile HashMap / Vec)  ──[Disconnected]──► SQLite Repositories (Dead code at runtime)
```

### After (Unified Deep Module)
```text
MainWindow / Client Service
    │
    ▼
ChatStore (Deep Module)
  ┌────────────────────────────────────────────────────────┐
  │ • Narrow Public API: get_chat, list_chats, messages,   │
  │   insert_message, upsert_chat, search                  │
  │ • Internal L1: Hot in-memory LRU cache                 │
  │ • Internal L2: SQLite write-through persistence        │
  └────────────────────────────────────────────────────────┘
```

---

## 3. Functional Requirements

- **Unified Surface:** `ChatStore` encapsulates connection pooling, database migrations, and caching.
- **Write-Through Persistence:** Inserting messages or syncing chats immediately updates the SQLite database.
- **Offline Startup:** On application launch, conversations load immediately from the local SQLite cache without waiting for daemon sync.

---

## 4. Acceptance Criteria

- [ ] `AppState` volatile hashmaps replaced by `ChatStore`.
- [ ] Messages and chats persist across client restarts.
- [ ] Full text search uses SQLite FTS / indexed queries through `ChatStore::search`.
- [ ] Unit tests for `ChatStore` achieve >85% coverage.
