# Specification: Wazzup Core MVP (GPUI WhatsApp Client with wacli Integration)

## 1. Overview
Wazzup is a GPU-accelerated desktop WhatsApp client built with Rust and Zed's GPUI framework. It communicates with a local `wacli` daemon/subprocess to synchronize contacts, chats, messages, and media, providing an ultra-responsive, lightweight Wayland chat client for Linux users.

## 2. Functional Requirements
### 2.1 Daemon Management & IPC
- Spawn and supervise `wacli` subprocess using `tokio::process::Command`.
- Stream and parse newline-delimited JSON-RPC / event messages from `wacli` stdout.
- Dispatch user commands (send message, fetch history, sync contacts, get media) via `wacli` CLI / stdin.
- Maintain and expose daemon connection status (Connecting, Paired, Disconnected, QR Code Pending).

### 2.2 Local Storage & Cache
- Initialize and maintain local SQLite database (`rusqlite`) for fast querying:
  - `chats`: JID, name, unread count, last message timestamp, pinned status.
  - `messages`: message ID, chat JID, sender JID, timestamp, body text, status (sent/delivered/read), media type/path.
  - `contacts`: JID, display name, phone number, avatar cache path.
- Provide query abstraction with pagination / limit-offset loading for message threads.

### 2.3 GPUI User Interface & Views
- **Application Shell:** GPU-rendered window with title bar, daemon status pill, search hotkey, and settings entry.
- **Sidebar (Chat List):** Virtualized list of active conversations with unread badges, timestamp formatting, last message preview, and selection highlighting.
- **Message Thread:** Scrollable message container with bubbles distinguishing incoming vs. outgoing messages, sender names in groups, timestamps, and delivery tick marks.
- **Message Input Composer:** Multiline text input supporting `Enter` to send, `Shift+Enter` for newline, emoji picker shortcut, and media attachment trigger.
- **Media Viewer:**
  - Inline image and sticker thumbnail rendering.
  - Overlay modal for full-size image viewing with zoom/pan and download button.
  - Audio message playback component with play/pause and progress scrubber using `symphonia` + `rodio`.

### 2.4 Navigation & Shortcuts
- Default desktop navigation: `Ctrl+K` for search palette, `Ctrl+N` for new chat, `Tab`/`Shift+Tab` for focus switching, `Arrow` keys for item selection, `Esc` to defocus.
- Optional Vim modal navigation toggleable in settings.
- Privacy mode shortcut (`Ctrl+Shift+P`) to blur all message previews and media.

## 3. Non-Functional Requirements
- **Performance:** Sub-10ms UI interaction latency; 60+ FPS smooth scrolling via GPUI.
- **Memory Footprint:** Baseline RSS < 150MB under regular operation.
- **Platform:** Native Wayland Linux support (tested on Hyprland / Omarchy).
- **Audio Decoding:** Zero external runtime binary dependencies for playback (pure Rust decoders via Symphonia).

## 4. Acceptance Criteria
- [ ] `wacli` subprocess can be started, monitored, and cleanly stopped.
- [ ] Real-time incoming JSON events from `wacli` update the local SQLite database and emit GPUI model events.
- [ ] Chat list renders contacts and updates immediately upon incoming message arrival.
- [ ] Outgoing message can be typed, dispatched to `wacli`, recorded in SQLite, and displayed in the thread.
- [ ] Image attachments decode and display properly in chat view.
- [ ] Audio voice notes can be played and paused without audio glitches or thread blocking.
- [ ] Unit tests for IPC parsing, database storage, and view model updates achieve >80% coverage.

## 5. Out of Scope for MVP
- Direct implementation of WhatsApp Noise/Signal protocol (delegated to `wacli`).
- Voice/video WebRTC calling inside GPUI.
- Multi-account switching (single active wacli session only).
