# Implementation Plan: Consolidate Reactive SQLite ChatStore

**Track ID:** `improve_sqlite_chat_store_20261016`  
**Type:** `refactor`  

---

## Phase 1: ChatStore Interface & In-Memory Engine (TDD)

- [x] Task: Define ChatStore Trait and Core Operations
  - [x] Write unit tests for `ChatStore` query, insertion, and pagination contracts (Red)
  - [x] Implement `ChatStore` with thread-safe SQLite connection manager (Green)
- [x] Task: Phase Verification & Checkpoint

## Phase 2: Write-Through Caching & Migration (TDD)

- [x] Task: Implement L1 Cache with SQLite L2 Persistence
  - [x] Write unit tests for cache hits, cache misses, and write-through persistence (Red)
  - [x] Implement write-through sync between memory and rusqlite repository (Green)
- [x] Task: Phase Verification & Checkpoint

## Phase 3: AppState & Window Migration

- [x] Task: Migrate AppState to ChatStore
  - [x] Replace volatile maps with `ChatStore` in `AppState`
  - [x] Verify startup offline restore and message search
- [x] Task: Phase Verification & Checkpoint
