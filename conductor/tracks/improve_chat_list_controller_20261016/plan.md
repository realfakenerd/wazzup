# Implementation Plan: Extract Focused ChatListController Component

**Track ID:** `improve_chat_list_controller_20261016`  
**Type:** `refactor`  

---

## Phase 1: Controller State & Navigation Logic (TDD)

- [ ] Task: Define ChatListState and Selection Operations
  - [ ] Write unit tests for list navigation indices, filter matching, and bounds safety (Red)
  - [ ] Implement `ChatListState` with selection and search query management (Green)
- [ ] Task: Phase Verification & Checkpoint

## Phase 2: GPUI Entity & Focus Integration (TDD)

- [ ] Task: Implement ChatListController with FocusHandle
  - [ ] Write unit tests for key action dispatch and selection event emission (Red)
  - [ ] Implement GPUI `Render` for `ChatListController` with `FocusHandle` (Green)
- [ ] Task: Phase Verification & Checkpoint

## Phase 3: MainWindow Refactor & Verification

- [ ] Task: Replace Inlined Sidebar in MainWindow
  - [ ] Mount `ChatListController` inside `MainWindow`
  - [ ] Subscribe to `ChatSelected` events in `MainWindow`
- [ ] Task: Phase Verification & Checkpoint
