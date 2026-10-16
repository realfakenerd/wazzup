# Implementation Plan: Non-Blocking Async WhatsApp Client Service Seam

**Track ID:** `improve_async_client_service_20261016`  
**Type:** `refactor`  

---

## Phase 1: Client Service Trait & Actor Channels (TDD)

- [ ] Task: Define WhatsAppClient Service Interface & Request/Response Types
  - [ ] Write unit tests for request/response serialization and actor message protocol (Red)
  - [ ] Implement `WhatsAppClient` trait and `MockWhatsAppClient` in-memory adapter (Green)
- [ ] Task: Phase Verification & Checkpoint

## Phase 2: Async Tokio Actor & Subprocess Engine (TDD)

- [ ] Task: Implement Background Async Actor
  - [ ] Write unit tests for actor request queueing, timeouts, and cancellation (Red)
  - [ ] Implement Tokio background worker bridging wacli CLI execution asynchronously (Green)
- [ ] Task: Phase Verification & Checkpoint

## Phase 3: MainWindow Integration & Zero-Block Verification

- [ ] Task: Wire Async Client into GPUI MainWindow
  - [ ] Connect `select_chat` to async background task dispatch with `cx.spawn`
  - [ ] Verify zero UI thread blocking during live chat switching
- [ ] Task: Phase Verification & Checkpoint
