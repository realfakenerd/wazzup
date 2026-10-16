# Track Specification: Extract Focused ChatListController Component

**Track ID:** `improve_chat_list_controller_20261016`  
**Type:** `refactor`  
**Recommendation Strength:** `Worth exploring` (`local-substitutable`)  

---

## 1. Overview & Context

`MainWindow` currently manages search string parsing, character-by-character mutation, index bounds checking, and item click closures inline across hundreds of lines of view code. The existing `render_chat_list` function remains a passive helper.

This track deepens the sidebar by extracting an autonomous `ChatListController` implementing GPUI `Render` with its own `FocusHandle`, dedicated key bindings (ArrowUp, ArrowDown, Esc), search filtering, and selection emission.

---

## 2. Before & After Architecture

### Before (Leaky Window Wiring)
```text
MainWindow
  ├── Inlined raw key handler (search query push/pop)
  ├── Inlined navigate_chat bounds math
  ├── Inlined mouse click listener creation
  └── Static layout render
```

### After (Autonomous Deep Controller)
```text
MainWindow
  └── ChatListController (GPUI Entity)
        ├── Owns FocusHandle
        ├── Handles list key events (Up/Down/Vim j/k)
        ├── Handles search query input & clear
        └── Emits ChatSelected(jid) to parent window
```

---

## 3. Functional Requirements

- **Autonomous Event Handling:** `ChatListController` captures focus and processes list-specific keyboard actions without leaking into global window scope.
- **Selection Emitter:** Selection of a chat emits a strongly-typed event (`ChatSelected(String)`) that parent views subscribe to.
- **Reusable Component:** The controller can be embedded into popups, search palettes, or alternative window layouts.

---

## 4. Acceptance Criteria

- [ ] `MainWindow` code reduced by extracting sidebar event handling into `ChatListController`.
- [ ] Keyboard navigation and search filtering work with localized `FocusHandle`.
- [ ] Unit tests for `ChatListController` state and keyboard interactions achieve >85% coverage.
