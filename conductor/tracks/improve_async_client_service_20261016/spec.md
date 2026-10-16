# Track Specification: Non-Blocking Async WhatsApp Client Service Seam

**Track ID:** `improve_async_client_service_20261016`  
**Type:** `refactor`  
**Recommendation Strength:** `Strong` (`ports & adapters`)  

---

## 1. Overview & Context

Currently, `MainWindow::select_chat` and `WacliLoader` synchronously spawn and block on `Command::new("wacli")` on the GPUI UI thread during user mouse clicks and initial launch. Under Wayland with 60–144Hz refresh rates, synchronous subprocess execution induces micro-stutters and frame drops.

This track introduces a deep `WhatsAppClient` service seam backed by a non-blocking Tokio actor that handles all subprocess communication off the main thread, dispatching typed events back to GPUI via async channels.

---

## 2. Before & After Architecture

### Before (Shallow & Synchronous)
```text
MainWindow (GPUI UI Thread)
    │ (blocks main thread)
    ▼
WacliLoader::fetch_chat_messages
    │ std::process::Command::new("wacli")
    ▼
wacli Subprocess
```

### After (Deep & Non-Blocking Seam)
```text
MainWindow (GPUI UI Thread)
    │ (non-blocking mpsc::Sender<ClientRequest>)
    ▼
WhatsAppClient (Tokio Actor)
    ├─► WacliSupervisor (Long-running JSON-lines Daemon)
    └─► Local Async Cache / DB
    │ (mpsc::Receiver<ClientResponse> -> cx.update())
    ▼
Reactive State Update in GPUI
```

---

## 3. Functional Requirements

- **Async Request Seam:** `WhatsAppClient` exposes async methods (`fetch_chats`, `fetch_messages`, `send_message`).
- **GPUI Integration:** Requests return background tasks that update the GPUI `Context<MainWindow>` asynchronously without stalling frame rendering.
- **Mock Seam:** Provide an in-memory `MockWhatsAppClient` for lightning-fast deterministic unit and UI testing without invoking `wacli`.

---

## 4. Acceptance Criteria

- [ ] Zero synchronous `std::process::Command` calls exist in UI rendering or event handlers.
- [ ] Chat switching is non-blocking with visual loading placeholder during history fetch.
- [ ] Unit tests for `WhatsAppClient` actor achieve >90% coverage with mock adapter.
