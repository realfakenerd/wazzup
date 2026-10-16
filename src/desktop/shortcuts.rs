#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    QuickSearch,
    NewChat,
    TogglePrivacy,
    ToggleVimMode,
    Escape,
    VimNextChat,
    VimPrevChat,
    VimEnterInsert,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSettings {
    pub privacy_mode: bool,
    pub vim_mode_enabled: bool,
    pub vim_mode: VimMode,
    pub search_palette_open: bool,
    pub new_chat_dialog_open: bool,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopSettings {
    pub fn new() -> Self {
        Self {
            privacy_mode: false,
            vim_mode_enabled: false,
            vim_mode: VimMode::Insert,
            search_palette_open: false,
            new_chat_dialog_open: false,
        }
    }

    pub fn toggle_privacy(&mut self) {
        self.privacy_mode = !self.privacy_mode;
    }

    pub fn toggle_vim_mode(&mut self) {
        self.vim_mode_enabled = !self.vim_mode_enabled;
        self.vim_mode = if self.vim_mode_enabled {
            VimMode::Normal
        } else {
            VimMode::Insert
        };
    }

    pub fn toggle_search_palette(&mut self) {
        self.search_palette_open = !self.search_palette_open;
        if self.search_palette_open {
            self.new_chat_dialog_open = false;
        }
    }

    pub fn toggle_new_chat(&mut self) {
        self.new_chat_dialog_open = !self.new_chat_dialog_open;
        if self.new_chat_dialog_open {
            self.search_palette_open = false;
        }
    }

    pub fn close_overlays(&mut self) {
        self.search_palette_open = false;
        self.new_chat_dialog_open = false;
        if self.vim_mode_enabled {
            self.vim_mode = VimMode::Normal;
        }
    }

    pub fn handle_keypress(
        &mut self,
        key: &str,
        ctrl: bool,
        shift: bool,
    ) -> Option<ShortcutAction> {
        // Global desktop key combinations
        if ctrl && !shift && key.eq_ignore_ascii_case("k") {
            self.toggle_search_palette();
            return Some(ShortcutAction::QuickSearch);
        }

        if ctrl && !shift && key.eq_ignore_ascii_case("n") {
            self.toggle_new_chat();
            return Some(ShortcutAction::NewChat);
        }

        if ctrl && shift && key.eq_ignore_ascii_case("p") {
            self.toggle_privacy();
            return Some(ShortcutAction::TogglePrivacy);
        }

        if ctrl && shift && key.eq_ignore_ascii_case("v") {
            self.toggle_vim_mode();
            return Some(ShortcutAction::ToggleVimMode);
        }

        if key == "Escape" || key == "esc" {
            self.close_overlays();
            return Some(ShortcutAction::Escape);
        }

        // Vim modal navigation when enabled
        if self.vim_mode_enabled {
            match self.vim_mode {
                VimMode::Normal => match key {
                    "j" => Some(ShortcutAction::VimNextChat),
                    "k" => Some(ShortcutAction::VimPrevChat),
                    "i" => {
                        self.vim_mode = VimMode::Insert;
                        Some(ShortcutAction::VimEnterInsert)
                    }
                    "/" => {
                        self.search_palette_open = true;
                        Some(ShortcutAction::QuickSearch)
                    }
                    _ => None,
                },
                VimMode::Insert => {
                    // In insert mode, typing is passed through to composer
                    None
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[core::prelude::v1::test]
    fn test_global_shortcuts() {
        let mut settings = DesktopSettings::new();

        // Ctrl+K -> Quick Search
        assert_eq!(
            settings.handle_keypress("k", true, false),
            Some(ShortcutAction::QuickSearch)
        );
        assert!(settings.search_palette_open);

        // Escape closes
        assert_eq!(
            settings.handle_keypress("Escape", false, false),
            Some(ShortcutAction::Escape)
        );
        assert!(!settings.search_palette_open);

        // Ctrl+Shift+P -> Privacy Toggle
        assert_eq!(
            settings.handle_keypress("p", true, true),
            Some(ShortcutAction::TogglePrivacy)
        );
        assert!(settings.privacy_mode);

        // Ctrl+N -> New Chat
        assert_eq!(
            settings.handle_keypress("n", true, false),
            Some(ShortcutAction::NewChat)
        );
        assert!(settings.new_chat_dialog_open);
    }

    #[core::prelude::v1::test]
    fn test_vim_modal_navigation() {
        let mut settings = DesktopSettings::new();
        settings.toggle_vim_mode();
        assert!(settings.vim_mode_enabled);
        assert_eq!(settings.vim_mode, VimMode::Normal);

        // Normal mode navigation: j/k
        assert_eq!(
            settings.handle_keypress("j", false, false),
            Some(ShortcutAction::VimNextChat)
        );
        assert_eq!(
            settings.handle_keypress("k", false, false),
            Some(ShortcutAction::VimPrevChat)
        );

        // Enter insert mode with 'i'
        assert_eq!(
            settings.handle_keypress("i", false, false),
            Some(ShortcutAction::VimEnterInsert)
        );
        assert_eq!(settings.vim_mode, VimMode::Insert);

        // In insert mode, j does nothing
        assert_eq!(settings.handle_keypress("j", false, false), None);

        // Escape returns to normal mode
        assert_eq!(
            settings.handle_keypress("Escape", false, false),
            Some(ShortcutAction::Escape)
        );
        assert_eq!(settings.vim_mode, VimMode::Normal);
    }
}
