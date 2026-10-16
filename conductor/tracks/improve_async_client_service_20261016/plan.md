# Implementation Plan: Non-Blocking Async WhatsApp Client Service Seam

**Track ID:** `improve_async_client_service_20261016`  
**Type:** `refactor`  

---

## Phase 1: Client Service Trait & Actor Channels (TDD)

- [x] Task: Define WhatsAppClient Service Interface & Request/Response Types
  - [x] Write unit tests for request/response serialization and actor message protocol (Red)
  - [x] Implement `WhatsAppClient` trait and `MockWhatsAppClient` in-memory adapter (Green)
- [x] Task: Phase Verification & Checkpoint

## Phase 2: Async Tokio Actor & Subprocess Engine (TDD)

- [x] Task: Implement Background Async Actor
  - [x] Write unit tests for actor request queueing, timeouts, and cancellation (Red)
  - [x] Implement Tokio background worker bridging wacli CLI execution asynchronously (Green)
- [x] Task: Phase Verification & Checkpoint

## Phase 3: MainWindow Integration & Zero-Block Verification

- [x] Task: Wire Async Client into GPUI MainWindow
  - [x] Connect `select_chat` to async background task dispatch with `cx.spawn`
  - [x] Verify zero UI thread blocking during live chat switching
- [x] Task: Phase Verification & Checkpoint
