# Technology Stack: Wazzup

## Core Runtime & Language
- **Language:** Rust (2024 Edition)
- **GUI Engine:** `gpui` (Zed Industries GPU framework via `git = "https://github.com/zed-industries/zed"`)
- **Async Runtime:** `tokio` (full features: async I/O, process management, mpsc/broadcast channels)

## Daemon & Protocol Integration
- **WhatsApp Engine:** `wacli` (WhatsApp CLI daemon and session controller)
- **Subprocess & IPC:** `tokio::process::Command` streaming line-delimited JSON-RPC / event messages from `wacli`
- **Data Serialization:** `serde` & `serde_json`

## Local Storage & Cache
- **Database:** `rusqlite` (bundled SQLite) for indexed message search, contact lists, conversation state, and media cache pointers
- **Paths & Directories:** `directories` crate adhering to XDG standards (`~/.config/wazzup`, `~/.cache/wazzup`, `~/.local/share/wazzup`)

## Media & Audio Pipeline
- **Image Processing:** `image` crate (PNG, JPEG, WebP decoding for avatars, chat images, and stickers)
- **Audio Playback:** `symphonia` (codec support for Opus, Ogg, AAC, MP3) + `rodio` (audio output stream)

## Desktop & OS Integration
- **Notifications:** `notify-rust` (Linux D-Bus desktop notifications)
- **Wayland Display:** Native Wayland compositor support handled via GPUI backends (Vulkan / EGL / Blade)
