use gpui::{div, px, rgb, Div, InteractiveElement, ParentElement, Styled};
use crate::db::repository::ChatRecord;
use crate::theme::ThemeColors;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatItemProps {
    pub jid: String,
    pub name: String,
    pub snippet: String,
    pub timestamp_str: String,
    pub unread_count: u32,
    pub is_selected: bool,
    pub is_pinned: bool,
}

impl ChatItemProps {
    pub fn from_record(record: &ChatRecord, snippet: Option<&str>, is_selected: bool) -> Self {
        let name = record
            .name
            .clone()
            .unwrap_or_else(|| format_jid_fallback(&record.jid));

        let timestamp_str = record
            .last_message_timestamp
            .map(format_timestamp)
            .unwrap_or_default();

        Self {
            jid: record.jid.clone(),
            name,
            snippet: snippet.unwrap_or("").to_string(),
            timestamp_str,
            unread_count: record.unread_count,
            is_selected,
            is_pinned: record.pinned,
        }
    }

    pub fn initials(&self) -> String {
        extract_initials(&self.name)
    }
}

pub fn extract_initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.as_slice() {
        [] => "?".to_string(),
        [single] => single.chars().take(2).collect::<String>().to_uppercase(),
        [first, .., last] => {
            let f = first.chars().next().unwrap_or('?');
            let l = last.chars().next().unwrap_or('?');
            format!("{}{}", f, l).to_uppercase()
        }
    }
}

pub fn format_jid_fallback(jid: &str) -> String {
    jid.split('@').next().unwrap_or(jid).to_string()
}

pub fn format_timestamp(timestamp: i64) -> String {
    // If timestamp is in seconds, convert or format as HH:MM
    let seconds_in_day = 86400;
    let time_of_day = (timestamp % seconds_in_day).abs();
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    format!("{:02}:{:02}", hours, minutes)
}

pub fn render_chat_item(props: &ChatItemProps) -> Div {
    let bg_color = if props.is_selected {
        rgb(ThemeColors::SURFACE0)
    } else {
        rgb(ThemeColors::BASE)
    };

    let initials = props.initials();

    div()
        .flex()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .w_full()
        .bg(bg_color)
        .hover(|s| s.bg(rgb(ThemeColors::SURFACE0)))
        .cursor_pointer()
        .border_b_1()
        .border_color(rgb(ThemeColors::BORDER))
        // Avatar circle
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_10()
                .h_10()
                .rounded_full()
                .bg(rgb(ThemeColors::SURFACE1))
                .text_color(rgb(ThemeColors::TEXT_PRIMARY))
                .text_size(px(14.0))
                .child(initials),
        )
        // Middle details: Name & snippet
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_1()
                .overflow_hidden()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(14.0))
                                .text_color(rgb(ThemeColors::TEXT_PRIMARY))
                                .child(props.name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(ThemeColors::TEXT_MUTED))
                                .child(props.timestamp_str.clone()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(ThemeColors::TEXT_MUTED))
                                .child(if props.snippet.is_empty() {
                                    "—".to_string()
                                } else {
                                    props.snippet.clone()
                                }),
                        )
                        .child(if props.unread_count > 0 {
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .min_w_5()
                                .h_5()
                                .px_1()
                                .rounded_full()
                                .bg(ThemeColors::emerald())
                                .text_size(px(11.0))
                                .text_color(rgb(ThemeColors::CRUST))
                                .child(props.unread_count.to_string())
                        } else {
                            div()
                        }),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[core::prelude::v1::test]
    fn test_extract_initials_single_word() {
        assert_eq!(extract_initials("Alice"), "AL");
    }

    #[core::prelude::v1::test]
    fn test_extract_initials_multiple_words() {
        assert_eq!(extract_initials("John Doe"), "JD");
        assert_eq!(extract_initials("Ana Maria Silva"), "AS");
    }

    #[core::prelude::v1::test]
    fn test_extract_initials_empty() {
        assert_eq!(extract_initials(""), "?");
    }

    #[core::prelude::v1::test]
    fn test_format_jid_fallback() {
        assert_eq!(
            format_jid_fallback("551199999999@s.whatsapp.net"),
            "551199999999"
        );
    }

    #[core::prelude::v1::test]
    fn test_format_timestamp() {
        let ts = 3600 * 14 + 60 * 30; // 14:30
        assert_eq!(format_timestamp(ts), "14:30");
    }

    #[core::prelude::v1::test]
    fn test_chat_item_props_from_record() {
        let record = ChatRecord {
            jid: "123@s.whatsapp.net".to_string(),
            name: Some("Alice".to_string()),
            unread_count: 3,
            last_message_timestamp: Some(3600 * 10 + 60 * 15),
            pinned: true,
        };

        let props = ChatItemProps::from_record(&record, Some("See you!"), true);
        assert_eq!(props.name, "Alice");
        assert_eq!(props.initials(), "AL");
        assert_eq!(props.snippet, "See you!");
        assert_eq!(props.timestamp_str, "10:15");
        assert_eq!(props.unread_count, 3);
        assert!(props.is_selected);
        assert!(props.is_pinned);
    }
}
