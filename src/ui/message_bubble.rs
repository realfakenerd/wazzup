use gpui::{div, px, rgb, Div, ParentElement, Styled};
use crate::db::repository::MessageRecord;
use crate::ipc::protocol::{MediaType, MessageStatus};
use crate::theme::ThemeColors;
use crate::ui::chat_item::format_timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBubbleProps {
    pub id: String,
    pub body: String,
    pub timestamp_str: String,
    pub from_me: bool,
    pub status: MessageStatus,
    pub media_info: Option<String>,
}

impl MessageBubbleProps {
    pub fn from_record(record: &MessageRecord) -> Self {
        let media_info = record.media.as_ref().map(|m| {
            let type_str = match m.media_type {
                MediaType::Image => "📷 Photo",
                MediaType::Audio => "🎵 Voice message",
                MediaType::Video => "🎥 Video",
                MediaType::Document => "📄 Document",
                MediaType::Sticker => "✨ Sticker",
            };
            match &m.file_name {
                Some(name) => format!("{} • {}", type_str, name),
                None => type_str.to_string(),
            }
        });

        Self {
            id: record.id.clone(),
            body: record.body.clone(),
            timestamp_str: format_timestamp(record.timestamp),
            from_me: record.from_me,
            status: record.status.clone(),
            media_info,
        }
    }

    pub fn status_tick_str(&self) -> &'static str {
        match self.status {
            MessageStatus::Pending => "🕒",
            MessageStatus::Sent => "✓",
            MessageStatus::Delivered => "✓✓",
            MessageStatus::Read => "✓✓",
            MessageStatus::Failed => "⚠️",
        }
    }

    pub fn status_tick_color(&self) -> gpui::Rgba {
        match self.status {
            MessageStatus::Read => rgb(0x53bdeb),      // Cyan read receipt
            MessageStatus::Failed => rgb(0xef4444),    // Red error
            _ => rgb(ThemeColors::TEXT_MUTED),        // Gray pending/sent/delivered
        }
    }
}

pub fn render_message_bubble(props: &MessageBubbleProps) -> Div {
    let (bubble_bg, align_justify) = if props.from_me {
        (
            rgb(0x005c4b), // WhatsApp Dark Outgoing Green
            div().justify_end(),
        )
    } else {
        (
            rgb(0x202c33), // WhatsApp Dark Incoming Surface
            div().justify_start(),
        )
    };

    let mut bubble_content = div()
        .max_w(px(520.0))
        .px_3()
        .py_2()
        .rounded_lg()
        .bg(bubble_bg)
        .border_color(rgb(ThemeColors::BORDER));

    // Media attachment preview header if present
    if let Some(ref media) = props.media_info {
        bubble_content = bubble_content.child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .pb_1()
                .text_size(px(12.0))
                .text_color(rgb(0x25d366))
                .child(media.clone()),
        );
    }

    // Message Text Body
    if !props.body.is_empty() {
        bubble_content = bubble_content.child(
            div()
                .text_size(px(13.5))
                .text_color(rgb(ThemeColors::TEXT_PRIMARY))
                .child(props.body.clone()),
        );
    }

    // Metadata footer: timestamp and ticks
    let mut footer = div()
        .flex()
        .items_center()
        .justify_end()
        .gap_1()
        .pt_1()
        .text_size(px(11.0))
        .text_color(rgb(ThemeColors::TEXT_MUTED))
        .child(props.timestamp_str.clone());

    if props.from_me {
        footer = footer.child(
            div()
                .text_color(props.status_tick_color())
                .child(props.status_tick_str()),
        );
    }

    bubble_content = bubble_content.child(footer);

    div()
        .flex()
        .w_full()
        .py_1()
        .px_4()
        .child(align_justify.w_full().flex().child(bubble_content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::protocol::MediaPayload;

    #[core::prelude::v1::test]
    fn test_tick_strings() {
        let make_props = |status: MessageStatus| MessageBubbleProps {
            id: "1".into(),
            body: "test".into(),
            timestamp_str: "10:00".into(),
            from_me: true,
            status,
            media_info: None,
        };

        assert_eq!(make_props(MessageStatus::Pending).status_tick_str(), "🕒");
        assert_eq!(make_props(MessageStatus::Sent).status_tick_str(), "✓");
        assert_eq!(make_props(MessageStatus::Delivered).status_tick_str(), "✓✓");
        assert_eq!(make_props(MessageStatus::Read).status_tick_str(), "✓✓");
        assert_eq!(make_props(MessageStatus::Failed).status_tick_str(), "⚠️");
    }

    #[core::prelude::v1::test]
    fn test_message_bubble_from_record() {
        let record = MessageRecord {
            id: "msg_42".to_string(),
            chat_jid: "chat@s.whatsapp.net".to_string(),
            sender_jid: "me@s.whatsapp.net".to_string(),
            timestamp: 3600 * 12,
            from_me: true,
            body: "Meeting at noon".to_string(),
            status: MessageStatus::Read,
            media: Some(MediaPayload {
                media_type: MediaType::Document,
                mimetype: "application/pdf".into(),
                file_path: Some("/tmp/agenda.pdf".into()),
                file_name: Some("agenda.pdf".into()),
                file_size: Some(1024),
            }),
        };

        let props = MessageBubbleProps::from_record(&record);
        assert_eq!(props.id, "msg_42");
        assert_eq!(props.body, "Meeting at noon");
        assert_eq!(props.timestamp_str, "12:00");
        assert!(props.from_me);
        assert_eq!(props.status, MessageStatus::Read);
        assert_eq!(props.media_info.as_deref(), Some("📄 Document • agenda.pdf"));
    }
}
