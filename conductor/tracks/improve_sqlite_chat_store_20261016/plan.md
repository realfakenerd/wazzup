# Implementation Plan: Consolidate Reactive SQLite ChatStore

**Track ID:** `improve_sqlite_chat_store_20261016`  
**Type:** `refactor`  

---

## Phase 1: ChatStore Interface & In-Memory Engine (TDD)

- [ ] Task: Define ChatStore Trait and Core Operations
  - [ ] Write unit tests for `ChatStore` query, insertion, and pagination contracts (Red)
  - [ ] Implement `ChatStore` with thread-safe SQLite connection manager (Green)
- [ ] Task: Phase Verification & Checkpoint

## Phase 2: Write-Through Caching & Migration (TDD)

- [ ] Task: Implement L1 Cache with SQLite L2 Persistence
  - [ ] Write unit tests for cache hits, cache misses, and write-through persistence (Red)
  - [ ] Implement write-through sync between memory and rusqlite repository (Green)
- [ ] Task: Phase Verification & Checkpoint

## Phase 3: AppState & Window Migration

- [ ] Task: Migrate AppState to ChatStore
  - [ ] Replace volatile maps with `ChatStore` in `AppState`
  - [ ] Verify startup offline restore and message search
- [ ] Task: Phase Verification & Checkpoint
