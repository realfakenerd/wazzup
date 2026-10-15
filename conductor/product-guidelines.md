# Product Guidelines: Wazzup

## 1. Voice & Tone
- Approachable, polished, and helpful UI copy.
- Clear error diagnostics and connection state reporting without obscure jargon.
- Informative tooltips and intuitive onboarding for wacli daemon pairing / QR scanning.

## 2. Visual Identity & Theming
- **Base Palette:** Omarchy / Zed Dark aesthetic (deep charcoal/slate backgrounds `#181825`, `#1e1e2e`).
- **Primary Accents:** Emerald WhatsApp green (`#25D366` / `#128C7E`) for active indicators, send buttons, and unread badges.
- **Message Indicators:** Classic WhatsApp tick marks (single tick = sent, double gray ticks = delivered, double cyan ticks `#34B7F1` = read).
- **Typography:** Clean modern sans-serif for UI copy and chat bubbles, combined with high-legibility monospace for technical details, timestamps, and search palettes.
- **Hardware Acceleration:** Fluid 60+ FPS layout animations and scroll inertia via GPUI.

## 3. Keyboard & Navigation Architecture
- **Default Navigation:** Standard modern desktop shortcuts (`Ctrl+K` command palette, `Ctrl+N` new chat, standard arrow keys, `Tab` / `Shift+Tab`, `Esc` to defocus).
- **Vim Mode (Optional):** Opt-in toggle in the settings menu enabling modal navigation (`j`/`k` chat cycling, `/` for search, normal/insert modal bindings).
- **Privacy Controls:** Quick toggle privacy blur mode (`Ctrl+Shift+P` / hotkey) to blur chat previews and media when stepping away.
- **Focus States:** Crisp, visible focus rings and keyboard-driven selection highlights.

## 4. Media & Interaction Principles
- **Inline Previews:** Lightweight preview cards embedded directly within chat bubbles for images, stickers, documents, and audio waveforms.
- **Fast Overlay Viewer:** Built-in GPU overlay modal for high-res image inspection, zoom/pan controls, audio scrubber, and quick-save hotkeys (`Ctrl+S`).
- **Status & Notifications:** Subtle status pill in the sidebar footer showing wacli sync and connection health; native Linux desktop notifications (libnotify/Freedesktop) with preview and sound toggles.
