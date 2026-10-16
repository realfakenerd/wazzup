# Specification: Lazy Background Chat History Backfill and Retrieval on Chat Selection

**Track ID:** `feat_lazy_history_retrieval_20261016`  
**Type:** `feature`  

---

## 1. Overview

Currently, selecting a conversation triggers `client.fetch_messages(&jid, 50)` asynchronously without querying local `ChatStore` messages first. The UI renders an empty thread until subprocess response finishes, causing visual flicker and latency. In addition, local query limit is hardcoded to 50, and older remote messages are never backfilled from the primary phone.

This track implements a two-stage lazy loading pattern:
1. **Instant Local Render:** On chat selection, immediately query `ChatStore` (0ms UI latency) and render available messages.
2. **Background Lazy Sync & Backfill:** Spawn background task through `WhatsAppClient` / `WacliActor`:
   - Step A: Run local fetch with extended limit (`--limit 1000`).
   - Step B: Run on-demand history backfill via `wacli history backfill --chat <jid> --count 50` to pull older batches from WhatsApp server/primary device.
   - Step C: Write newly discovered messages into `ChatStore` write-through storage.
   - Step D: Stream updated thread back to UI thread and notify GPUI context.
3. **In-Flight Deduplication:** Avoid redundant subprocess spawns if a backfill for the given `chat_jid` is already running.

---

## 2. Functional Requirements

### 2.1 Instant Local Rendering (0ms UI Latency)
- `AppState::select_chat` or `MainWindow::select_chat` must populate `messages_by_chat` directly from `ChatStore::get_messages(jid, 1000)` synchronously if not already present.
- Thread view displays stored messages immediately without waiting for IPC or network round-trip.

### 2.2 Client Trait & WacliActor History Extension
- Extend `WhatsAppClient` trait and `ChannelWhatsAppClient` with `sync_chat_history(&self, chat_jid: &str, backfill_count: u32) -> BoxFuture<'_, Result<Vec<MessageRecord>, String>>`.
- Add `ClientRequest::SyncChatHistory { chat_jid: String, backfill_count: u32 }` to IPC protocol.
- In `WacliActor`:
  - Execute `wacli messages list --chat <jid> --limit 1000 --json` (replacing hard-coded 50 cap).
  - Execute `wacli history backfill --chat <jid> --count <backfill_count>` when requested.
  - Return consolidated, timestamp-sorted `Vec<MessageRecord>`.

### 2.3 Write-Through SQLite Persistence
- Newly retrieved messages from background sync are persisted into `ChatStore` via `insert_message` or batch insert.
- `ChatStore` updates L1 cache and SQLite L2 storage while updating the chat's `last_message_timestamp`.

### 2.4 In-Flight Deduplication & Idempotency
- Track active backfill operations by JID using an in-flight set (`Arc<Mutex<HashSet<String>>>` or actor-level guard).
- If selection occurs while a sync for that `chat_jid` is already active, ignore redundant dispatch.
- Ensure in-flight state cleans up on success, failure, or timeout.

---

## 3. Non-Functional Requirements & Invariants

- **Zero UI Block:** All subprocess invocations (`wacli messages list`, `wacli history backfill`) run asynchronously inside Tokio runtime. Main thread never blocks on I/O.
- **Zero Visual Render Tests Constraint:** Per user constraint, all verification is strictly logic, state transitions, repository queries, and actor IPC channels. Zero GPUI element tree rendering tests.
- **Graceful Degradation:** If `wacli history backfill` returns error (e.g. offline, rate-limited, primary phone unreachable), actor logs warning and falls back to returning messages from local list.

---

## 4. Acceptance Criteria

- [ ] Chat selection immediately displays cached SQLite messages prior to any subprocess response.
- [ ] `WacliActor` handles `SyncChatHistory` request, executing `--limit 1000` list query and `--count 50` backfill.
- [ ] Repeated rapid clicks on the same chat do not spawn duplicate concurrent backfill subprocesses.
- [ ] Freshly retrieved messages persist into SQLite database without duplicates.
- [ ] Logic/state/IPC unit tests for `WacliActor`, `WhatsAppClient`, and `ChatStore` achieve 100% pass rate.

---

## 5. Out of Scope

- Infinite scrolling trigger for historical backfill beyond chat selection.
- Visual GPUI layout/styling tests.
- Multi-account concurrent sync.
