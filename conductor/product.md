# Product Definition: Wazzup

## Overview
**Wazzup** is a blazing-fast, GPU-accelerated desktop WhatsApp client engineered in Rust using Zed's **GPUI** framework. It acts as a lightweight, native GUI frontend driven by **wacli** (WhatsApp CLI/daemon), offering instant responsiveness, minimal memory footprint, and tight integration with modern Linux Wayland environments (Hyprland / Omarchy).

## Problem Statement
Existing WhatsApp desktop clients (Electron-based wrappers, web apps, or heavy browser tabs) suffer from high RAM consumption, sluggish startup times, high input latency, and poor integration with keyboard-driven Linux tiling window managers. Users on lightweight or developer setups need a fluid, native, and keyboard-friendly WhatsApp client that runs efficiently.

## Target Audience & Environment
- **Primary Platform:** Linux / Wayland (optimized for Hyprland and Omarchy desktop environments).
- **Target Users:** Developers, keyboard-centric power users, and Linux desktop enthusiasts seeking a lean, aesthetic WhatsApp client with near-zero latency.
- **Display & Windowing:** Fully GPU-rendered via GPUI, keyboard-navigable, with native desktop notification integration.

## Key Capabilities & MVP Scope
1. **Real-time Chat Management:**
   - Active chat list with unread badges, last message previews, and timestamp ordering.
   - Message thread viewer with scrollback virtualization and real-time incoming message updates.
   - Outgoing text messaging with delivery status indicators.
2. **Rich Media Support (MVP):**
   - In-app rendering and viewing of media: images, stickers, and document attachments.
   - Audio message playback support.
3. **`wacli` Subprocess & IPC Integration:**
   - Robust subprocess management wrapping `wacli` CLI / daemon.
   - Structured JSON IPC and streaming stdout event parsing for real-time state synchronization.
   - Visible daemon health, connection state, and QR-code/pairing session management.
4. **Visual Design & User Experience:**
   - Dark, refined Zed / Omarchy aesthetic with hardware-accelerated fluid rendering.
   - Clean monospace / modern sans typography.
   - Keyboard shortcuts for rapid chat switching, navigation, and message composition.

## Non-Goals & Future Iterations
- Direct reimplementation of WhatsApp multi-device encryption protocols in the GUI layer (delegated entirely to `wacli`).
- Heavy voice/video call WebRTC rendering inside the MVP (targeted for post-MVP).
