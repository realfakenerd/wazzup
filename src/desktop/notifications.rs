use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationPayload {
    pub summary: String,
    pub body: String,
    pub app_name: String,
}

pub struct NotificationRouter;

impl NotificationRouter {
    /// Determines whether an incoming message notification should be silenced.
    /// A notification is suppressed only if the window is currently focused AND
    /// the message belongs to the currently active conversation.
    pub fn should_suppress(is_window_focused: bool, is_active_chat: bool) -> bool {
        is_window_focused && is_active_chat
    }

    /// Format summary and body for the desktop notification daemon.
    /// If privacy mode is active, the message body is obscured to prevent shoulder surfing.
    pub fn format_notification(
        sender_name: &str,
        message_body: &str,
        is_privacy_mode: bool,
    ) -> NotificationPayload {
        let summary = if sender_name.trim().is_empty() {
            "New WhatsApp Message".to_string()
        } else {
            sender_name.trim().to_string()
        };

        let body = if is_privacy_mode {
            "••••••••••••".to_string()
        } else if message_body.trim().is_empty() {
            "Attachment".to_string()
        } else {
            message_body.trim().to_string()
        };

        NotificationPayload {
            summary,
            body,
            app_name: "Wazzup".to_string(),
        }
    }

    /// Dispatch desktop notification via native freedesktop / notify-send daemon
    pub fn dispatch(payload: &NotificationPayload) -> std::io::Result<()> {
        Command::new("notify-send")
            .arg("-a")
            .arg(&payload.app_name)
            .arg(&payload.summary)
            .arg(&payload.body)
            .spawn()?
            .wait()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[core::prelude::v1::test]
    fn test_suppression_logic() {
        // Suppress when both focused and active
        assert!(NotificationRouter::should_suppress(true, true));

        // Do not suppress if unfocused (even if active chat)
        assert!(!NotificationRouter::should_suppress(false, true));

        // Do not suppress if focused but message from another chat
        assert!(!NotificationRouter::should_suppress(true, false));

        // Do not suppress if both unfocused and inactive
        assert!(!NotificationRouter::should_suppress(false, false));
    }

    #[core::prelude::v1::test]
    fn test_format_notification_normal() {
        let payload = NotificationRouter::format_notification("Alice", "Are you ready?", false);
        assert_eq!(payload.summary, "Alice");
        assert_eq!(payload.body, "Are you ready?");
        assert_eq!(payload.app_name, "Wazzup");
    }

    #[core::prelude::v1::test]
    fn test_format_notification_privacy_mode() {
        let payload = NotificationRouter::format_notification("Alice", "Top secret password", true);
        assert_eq!(payload.summary, "Alice");
        assert_eq!(payload.body, "••••••••••••");
    }

    #[core::prelude::v1::test]
    fn test_format_notification_empty_sender_fallback() {
        let payload = NotificationRouter::format_notification("", "Hello", false);
        assert_eq!(payload.summary, "New WhatsApp Message");
        assert_eq!(payload.body, "Hello");
    }
}
