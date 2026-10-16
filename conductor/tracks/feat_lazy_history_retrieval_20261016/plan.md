# Implementation Plan: Lazy Background Chat History Backfill and Retrieval on Chat Selection

**Track ID:** `feat_lazy_history_retrieval_20261016`  
**Type:** `feature`  

---

## Phase 1: ChatStore Instant Message Loading & Deduplication (TDD)

- [x] Task: Instant Local Loading on Chat Selection
  - [x] Write unit test in `state.rs` verifying `select_chat` populates `messages_by_chat` from `ChatStore` (Red)
  - [x] Update `AppState::select_chat` to query `ChatStore::get_messages(jid, 1000)` on selection (Green)
  - [x] Write unit test for batch message upsert in `ChatStore` avoiding duplicate IDs and keeping sorting (Red)
  - [x] Implement batch insertion / idempotent message upsert in `ChatStore` (Green)
- [x] Task: Phase Verification & Checkpoint

---

## Phase 2: WhatsAppClient Trait & WacliActor History Backfill (TDD)

- [x] Task: IPC Protocol and Trait Extension
  - [x] Add `ClientRequest::SyncChatHistory` and update trait with `sync_chat_history`
  - [x] Write unit test in `client.rs` verifying `MockWhatsAppClient` and `ChannelWhatsAppClient` dispatch for `SyncChatHistory` (Red)
  - [x] Implement `sync_chat_history` on `MockWhatsAppClient` and `ChannelWhatsAppClient` (Green)
- [x] Task: WacliActor Subprocess Pipeline
  - [x] Write unit test in `actor.rs` verifying `SyncChatHistory` dispatches list and backfill commands (Red)
  - [x] Implement `fetch_chat_messages_async` with configurable limit up to 1000 (Green)
  - [x] Implement `backfill_history_async` in `WacliActor` invoking `wacli history backfill` (Green)
- [x] Task: Phase Verification & Checkpoint

---

## Phase 3: MainWindow In-Flight Deduplication & Background Sync Dispatch (TDD)

- [x] Task: In-Flight Sync Tracking
  - [x] Write unit test for `InFlightSyncTracker` ensuring concurrent duplicate JID syncs are rejected (Red)
  - [x] Implement thread-safe in-flight tracking set (Green)
- [x] Task: Wire MainWindow select_chat_async
  - [x] Update `MainWindow::select_chat_async` to query store instantly, check in-flight deduplication, and spawn `sync_chat_history` (Green)
  - [x] Write unit test in `window.rs` / `state.rs` asserting instant store loading and async message merge without UI render tests (Red -> Green)
- [x] Task: Phase Verification & Checkpoint

---

## Phase 4: Integration Verification Checkpoint

- [x] Task: Full Pure-Logic Test Suite Execution
  - [x] Run `cargo test --lib` verifying IPC, actor, state, and store test pass
  - [x] Verify `conductor verify` reports 0 broken links or orphan tracks
- [x] Task: Final Track Checkpoint
