# Implementation Plan: Wazzup Core MVP

## Phase 1: Core Foundation & Subprocess IPC Engine [checkpoint: 79c9272]

- [x] Task: Project Scaffolding & Dependencies Setup [7dcd1b5]
  - [ ] Add GPUI (Zed git repo), tokio, serde, serde_json, rusqlite, notify-rust to Cargo.toml
  - [ ] Configure linting and clippy settings
  - [ ] Verify cargo check and cargo test build cleanly
- [x] Task: wacli IPC Protocol Models (TDD) [c711db4]
  - [ ] Write unit tests for wacli JSON-RPC event message deserialization (Red)
  - [ ] Implement event data types (ChatEvent, MessageEvent, StatusEvent) with Serde (Green)
  - [ ] Refactor serialization schemas for zero-copy parsing where applicable
- [x] Task: Subprocess Daemon Supervisor (TDD) [a1aea00]
  - [ ] Write unit tests for daemon process spawn, heartbeat, and exit handling with mock process (Red)
  - [ ] Implement Tokio async subprocess runner streaming stdout lines into broadcast channels (Green)
  - [ ] Implement command dispatching to wacli (send text, query status) (Green)
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) [79c9272]

## Phase 2: Local SQLite Cache & Persistence Layer

- [ ] Task: Database Schema & Migration Setup (TDD)
  - [ ] Write unit tests for database schema initialization and table creation (Red)
  - [ ] Implement rusqlite schema creation for chats, messages, and contacts (Green)
- [ ] Task: Message & Chat Repository (TDD)
  - [ ] Write unit tests for inserting, updating, and paginating chats and message threads (Red)
  - [ ] Implement ChatRepository and MessageRepository with transactional updates (Green)
  - [ ] Write unit tests for full-text search across local message cache (Red)
  - [ ] Implement SQLite FTS / LIKE search querying (Green)
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 3: GPUI Application Shell & State Models

- [ ] Task: GPUI Window & Theme Initialization
  - [ ] Scaffolding GPUI App entrypoint with Omarchy/Zed Dark palette tokens
  - [ ] Configure Wayland window options and titlebar
- [ ] Task: App State & Reactive Store (TDD)
  - [ ] Write unit tests for AppState model, active chat selection, and event listener dispatch (Red)
  - [ ] Implement GPUI Model<AppState> bridging tokio background event channel into GPUI UI loop (Green)
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 4: Chat List & Message Thread Views

- [ ] Task: Chat List Sidebar Component
  - [ ] Implement ChatItemView with unread badge, last message preview, and timestamp
  - [ ] Implement virtualized ChatListView with keyboard selection
- [ ] Task: Message Thread & Chat Bubble Components
  - [ ] Implement MessageBubbleView distinguishing incoming vs outgoing messages
  - [ ] Add WhatsApp delivery tick indicators (single, double gray, double cyan)
  - [ ] Implement auto-scroll to bottom on new incoming message
- [ ] Task: Message Composer Component
  - [ ] Implement text input area with Enter-to-send and Shift+Enter newline
  - [ ] Connect send action to wacli daemon IPC and local database update
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 5: Rich Media Support & Audio Playback

- [ ] Task: Image & Sticker Thumbnail Renderer (TDD)
  - [ ] Write unit tests for image dimensions and caching helper (Red)
  - [ ] Implement image decoding via image crate and rendering in GPUI view (Green)
  - [ ] Add overlay modal for full-size inspection and zoom
- [ ] Task: Audio Player Engine (TDD)
  - [ ] Write unit tests for Symphonia audio decoder and Rodio player wrapper (Red)
  - [ ] Implement non-blocking audio playback with playback progress reporting (Green)
  - [ ] Implement GPUI AudioPlayerView with play/pause button and seekbar
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 6: Desktop Integration & Final Polish

- [ ] Task: Native Desktop Notifications (TDD)
  - [ ] Write unit tests for notification formatting and suppression when window is focused (Red)
  - [ ] Implement notify-rust integration with desktop notification routing (Green)
- [ ] Task: Keyboard Shortcuts & Privacy Mode
  - [ ] Implement Ctrl+K quick search palette, Ctrl+N new chat
  - [ ] Implement privacy mode blur toggle (Ctrl+Shift+P)
  - [ ] Add settings toggle for optional Vim modal navigation
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)







