# Implementation Plan: Interactive Native TextInput Entity and Virtualized UniformList Rendering

## Phase 1: Dependencies & Core TextInput Model Logic
- [x] Task 1.1: Add `unicode-segmentation = "1.13"` to `Cargo.toml`.
- [x] Task 1.2: Implement `TextInput` data structures, UTF-16/UTF-8 index conversions, grapheme boundaries, and range manipulations in `src/ui/text_input.rs`.
- [x] Task 1.3: Add pure-logic unit tests for cursor movement, grapheme boundaries, and range replacement in `src/ui/text_input.rs`.
- [x] Phase 1 Verification Checkpoint: Run `cargo test --offline` to verify logic unit tests pass.

## Phase 2: GPUI EntityInputHandler & TextElement Implementation
- [x] Task 2.1: Implement `EntityInputHandler` and `Focusable` on `TextInput`.
- [x] Task 2.2: Implement `TextElement` (GPUI `Element` trait with `request_layout`, `prepaint`, and `paint`) for text shaping, selection quads, and cursor drawing.
- [x] Task 2.3: Implement `Render` for `TextInput` with action listeners (`Backspace`, `Delete`, `Left`, `Right`, `SelectAll`, `Cut`, `Copy`, `Paste`).
- [x] Phase 2 Verification Checkpoint: Compile with `cargo check --offline`.

## Phase 3: Integration into Composer & Search Bar
- [x] Task 3.1: Integrate `TextInput` into `ComposerState` and `render_composer`.
- [x] Task 3.2: Integrate `TextInput` into `ChatListController` search bar.
- [x] Task 3.3: Wire Vim mode focus switching (Vim Normal blurs text input, Vim Insert focuses text input).
- [x] Phase 3 Verification Checkpoint: Run `cargo test --offline` and verify logic tests pass.

## Phase 4: Virtualized Scrollback with UniformList
- [x] Task 4.1: Integrate `uniform_list` for chat list rendering in `ChatListController`.
- [x] Task 4.2: Keep variable-height message thread history on `overflow_y_scroll()` while chat list uniform rows are virtualized with `uniform_list`.
- [x] Task 4.3: Add unit tests for list indexing and range calculation logic (`test_build_chat_item_props`).
- [x] Phase 4 Verification Checkpoint: Full test run and binary build `cargo test --offline && cargo build --offline`.
