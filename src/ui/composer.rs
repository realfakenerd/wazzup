use gpui::{div, px, rgb, Div, Entity, InteractiveElement, ParentElement, Styled};
use crate::ipc::protocol::WacliCommand;
use crate::theme::ThemeColors;
use crate::ui::text_input::TextInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerState {
    pub text: String,
    pub is_sending: bool,
}

impl Default for ComposerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ComposerState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            is_sending: false,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.is_sending = false;
    }

    pub fn create_send_command(&self, to_jid: &str) -> Option<WacliCommand> {
        let trimmed = self.text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(WacliCommand::SendText {
                to: to_jid.to_string(),
                text: trimmed.to_string(),
            })
        }
    }
}

pub fn render_composer(
    state: &ComposerState,
    has_active_chat: bool,
    text_input: Option<&Entity<TextInput>>,
) -> Div {
    if !has_active_chat {
        return div();
    }

    let is_send_enabled = (!state.text.trim().is_empty() || text_input.is_some()) && !state.is_sending;

    div()
        .flex()
        .items_center()
        .h(px(56.0))
        .px_4()
        .gap_3()
        .bg(ThemeColors::mantle())
        .border_t_1()
        .border_color(rgb(ThemeColors::BORDER))
        // Attachment Button
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_9()
                .h_9()
                .rounded_full()
                .hover(|s| s.bg(rgb(ThemeColors::SURFACE0)))
                .cursor_pointer()
                .text_color(ThemeColors::text_muted())
                .text_size(px(18.0))
                .child("📎"),
        )
        // Text Input Field Container
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .h_10()
                .px_4()
                .rounded_lg()
                .bg(rgb(ThemeColors::SURFACE0))
                .border_1()
                .border_color(rgb(ThemeColors::BORDER))
                .child(if let Some(input) = text_input {
                    div().flex_1().child(input.clone())
                } else {
                    div()
                        .text_size(px(14.0))
                        .text_color(if state.text.is_empty() {
                            ThemeColors::text_muted()
                        } else {
                            ThemeColors::text_primary()
                        })
                        .child(if state.text.is_empty() {
                            "Type a message (Enter to send)...".to_string()
                        } else {
                            state.text.clone()
                        })
                }),
        )
        // Send Button
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_10()
                .h_10()
                .rounded_full()
                .bg(if is_send_enabled {
                    ThemeColors::emerald()
                } else {
                    rgb(ThemeColors::SURFACE1)
                })
                .text_color(if is_send_enabled {
                    rgb(ThemeColors::CRUST)
                } else {
                    ThemeColors::text_muted()
                })
                .text_size(px(16.0))
                .cursor_pointer()
                .child(if state.is_sending { "⏳" } else { "➤" }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[core::prelude::v1::test]
    fn test_composer_state_empty_send_is_none() {
        let composer = ComposerState::new();
        assert!(composer.create_send_command("123@s.whatsapp.net").is_none());
    }

    #[core::prelude::v1::test]
    fn test_composer_state_creates_send_text_command() {
        let mut composer = ComposerState::new();
        composer.set_text("   Hello there!   ");

        let cmd = composer.create_send_command("123@s.whatsapp.net");
        assert!(cmd.is_some());
        match cmd.unwrap() {
            WacliCommand::SendText { to, text } => {
                assert_eq!(to, "123@s.whatsapp.net");
                assert_eq!(text, "Hello there!");
            }
            _ => panic!("Expected SendText command"),
        }
    }

    #[core::prelude::v1::test]
    fn test_composer_clear() {
        let mut composer = ComposerState::new();
        composer.set_text("Message");
        composer.is_sending = true;
        composer.clear();
        assert!(composer.text.is_empty());
        assert!(!composer.is_sending);
    }
}
