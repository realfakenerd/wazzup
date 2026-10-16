# Specification: Interactive Native TextInput Entity and Virtualized UniformList Rendering

## Overview
Based on GPUI canonical patterns from `gpui/examples/input.rs` and `gpui/examples/uniform_list.rs`, this track upgrades Wazzup from primitive keystroke string append logic to first-class GPUI primitives:
1. A reusable `TextInput` entity implementing `EntityInputHandler` and `Focusable`, providing full cursor navigation, text selection, system clipboard, and native Wayland IME text composition for the message composer and search bar.
2. High-performance viewport virtualization using GPUI's `uniform_list` for chat list and message scrollback rendering, preventing frame drops and memory spikes when handling thousands of messages.

## Functional Requirements
- **FR-1 (TextInput State & Editing)**: Provide `TextInput` struct supporting cursor placement, word/grapheme boundary navigation (`Left`, `Right`, `Home`, `End`), deletion (`Backspace`, `Delete`), selection (`SelectLeft`, `SelectRight`, `SelectAll`), and clipboard (`Cut`, `Copy`, `Paste`).
- **FR-2 (IME & Input Handler)**: Implement `EntityInputHandler` on `TextInput` to interface with Wayland `zwp_text_input_v3` for accented characters, dead keys, and composition markings.
- **FR-3 (Visual Element Rendering)**: Render text via `TextElement` implementing GPUI `Element`, displaying shaped text runs, active selection highlights, and blinking cursor quad when focused.
- **FR-4 (Composer Integration)**: Upgrade `render_composer` to house the `TextInput` entity, dispatching `SendMessage` on `Enter` while retaining normal text typing.
- **FR-5 (Search Bar Integration)**: House a `TextInput` entity inside `ChatListController` search header, triggering live filtered query updates on change and clearing on `Escape`.
- **FR-6 (UniformList Virtualization)**: Virtualize large lists using `uniform_list` to render only elements within viewport bounds.

## Non-Functional Requirements
- **Performance**: Zero UI blocking; sub-millisecond cursor movement; constant 60fps scrolling even with 1,000+ cached messages.
- **Testing Constraint**: Invariant rule: *"I want you to make tests for logic only, let the visual tests be made by me"*. Test string manipulation, UTF-16 math, selection ranges, cursor offsets, and list calculations purely in Rust unit tests.

## Acceptance Criteria
- [ ] `TextInput` correctly calculates cursor offsets, grapheme cluster boundaries, and UTF-16/UTF-8 conversions.
- [ ] Text replacement, insertion, deletion, and selection ranges pass all logic unit tests.
- [ ] Composer and Search Bar utilize `TextInput`.
- [ ] `uniform_list` virtualizes rendering in chat list and message thread.
- [ ] All unit tests pass with `cargo test --offline`.
- [ ] Binary compiles cleanly with `cargo build --offline`.
