# Implementation Plan: Extract Focused ChatListController Component

**Track ID:** `improve_chat_list_controller_20261016`  
**Type:** `refactor`  

---

## Phase 1: Controller State & Navigation Logic (TDD)

- [x] Task: Define ChatListState and Selection Operations
  - [x] Write unit tests for list navigation indices, filter matching, and bounds safety (Red)
  - [x] Implement `ChatListState` with selection and search query management (Green)
- [x] Task: Phase Verification & Checkpoint

## Phase 2: GPUI Entity & Focus Integration (TDD)

- [x] Task: Implement ChatListController with FocusHandle
  - [x] Write unit tests for key action dispatch and selection event emission (Red)
  - [x] Implement GPUI `Render` for `ChatListController` with `FocusHandle` (Green)
- [x] Task: Phase Verification & Checkpoint

## Phase 3: MainWindow Refactor & Verification

- [x] Task: Replace Inlined Sidebar in MainWindow
  - [x] Mount `ChatListController` inside `MainWindow`
  - [x] Subscribe to `ChatSelected` events in `MainWindow`
- [x] Task: Phase Verification & Checkpoint
