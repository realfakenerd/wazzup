# Implementation Plan: Lazy Background Chat History Backfill and Retrieval on Chat Selection

**Track ID:** `feat_lazy_history_retrieval_20261016`  
**Type:** `feature`  

---

## Phase 1: ChatStore Instant Message Loading & Deduplication (TDD)

- [ ] Task: Instant Local Loading on Chat Selection
  - [ ] Write unit test in `state.rs` verifying `select_chat` populates `messages_by_chat` from `ChatStore` (Red)
  - [ ] Update `AppState::select_chat` to query `ChatStore::get_messages(jid, 1000)` on selection (Green)
  - [ ] Write unit test for batch message upsert in `ChatStore` avoiding duplicate IDs and keeping sorting (Red)
  - [ ] Implement batch insertion / idempotent message upsert in `ChatStore` (Green)
- [ ] Task: Phase Verification & Checkpoint

---

## Phase 2: WhatsAppClient Trait & WacliActor History Backfill (TDD)

- [ ] Task: IPC Protocol and Trait Extension
  - [ ] Add `ClientRequest::SyncChatHistory` and update trait with `sync_chat_history`
  - [ ] Write unit test in `client.rs` verifying `MockWhatsAppClient` and `ChannelWhatsAppClient` dispatch for `SyncChatHistory` (Red)
  - [ ] Implement `sync_chat_history` on `MockWhatsAppClient` and `ChannelWhatsAppClient` (Green)
- [ ] Task: WacliActor Subprocess Pipeline
  - [ ] Write unit test in `actor.rs` verifying `SyncChatHistory` dispatches list and backfill commands (Red)
  - [ ] Implement `fetch_chat_messages_async` with configurable limit up to 1000 (Green)
  - [ ] Implement `backfill_history_async` in `WacliActor` invoking `wacli history backfill` (Green)
- [ ] Task: Phase Verification & Checkpoint

---

## Phase 3: MainWindow In-Flight Deduplication & Background Sync Dispatch (TDD)

- [ ] Task: In-Flight Sync Tracking
  - [ ] Write unit test for `InFlightSyncTracker` ensuring concurrent duplicate JID syncs are rejected (Red)
  - [ ] Implement thread-safe in-flight tracking set (Green)
- [ ] Task: Wire MainWindow select_chat_async
  - [ ] Update `MainWindow::select_chat_async` to query store instantly, check in-flight deduplication, and spawn `sync_chat_history` (Green)
  - [ ] Write unit test in `window.rs` / `state.rs` asserting instant store loading and async message merge without UI render tests (Red -> Green)
- [ ] Task: Phase Verification & Checkpoint

---

## Phase 4: Integration Verification Checkpoint

- [ ] Task: Full Pure-Logic Test Suite Execution
  - [ ] Run `cargo test --lib` verifying IPC, actor, state, and store test pass
  - [ ] Verify `conductor verify` reports 0 broken links or orphan tracks
- [ ] Task: Final Track Checkpoint
