# Wazzup

> GPU-accelerated WhatsApp desktop client for Linux/Wayland written in Rust with Zed's GPUI framework.

Wazzup delivers a native, blisteringly fast desktop experience for WhatsApp on Linux. Instead of running a heavy Electron wrapper consuming hundreds of megabytes of RAM, Wazzup renders 60fps+ hardware-accelerated UI via Vulkan/OpenGL shaders and connects to WhatsApp through `wacli` daemon IPC.

---

## Highlights

- **GPU Accelerated:** Driven by Zed's [GPUI](https://github.com/zed-industries/zed) UI framework with native Wayland windowing.
- **Ultra-light Footprint:** Negligible idle memory and CPU consumption compared to Electron/Chromium wrappers.
- **Daemon-isolated IPC:** Communicates with `wacli` over robust Tokio-supervised JSON-lines streams.
- **Local Cache & Offline Search:** Fast local SQLite database (`rusqlite`) for instant search across chats and messages without network roundtrips.
- **Linux First:** Tailored for Wayland compositors (Hyprland, Sway, GNOME, KDE) with dark palette tokens matching modern desktop environments.

---

## Architecture

```text
┌────────────────────────────────────────────────────────┐
│                   Wazzup GUI (GPUI)                    │
│   • MainWindow / ChatList / MessageThread / Composer   │
│   • Vulkan / Wayland surface rendering                 │
└──────────────────────────┬─────────────────────────────┘
                           │ Async Event Channels (Tokio)
┌──────────────────────────┴─────────────────────────────┐
│                    Core Engine                         │
│   • State Management                                   │
│   • SQLite Local Storage (chats, messages, media)      │
│   • wacli Subprocess Supervisor                        │
└──────────────────────────┬─────────────────────────────┘
                           │ JSON-Lines IPC (stdin/stdout)
┌──────────────────────────┴─────────────────────────────┐
│                     wacli Daemon                       │
│   • WhatsApp Multi-Device protocol session             │
└────────────────────────────────────────────────────────┘
```

---

## Requirements

- **Linux** (Wayland recommended)
- **Rust Toolchain:** Rust 1.85+ (2024 edition)
- **`wacli`:** WhatsApp CLI tool configured and paired on your machine
- **System Dependencies:**
  - `pkg-config`
  - `libxkbcommon`
  - `vulkan-loader` (or `mesa` drivers)
  - `sqlite3`

---

## Getting Started

### 1. Clone repository

```bash
git clone https://github.com/realfakenerd/wazzup.git
cd wazzup
```

### 2. Run checks / tests

```bash
cargo check
cargo test --lib
```

### 3. Run client

```bash
cargo run
```

---

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
