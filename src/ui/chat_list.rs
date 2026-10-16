use gpui::{div, px, rgb, Div, ParentElement, Styled};
use crate::db::repository::ChatRecord;
use crate::theme::ThemeColors;
use crate::ui::chat_item::{render_chat_item, ChatItemProps};

pub fn render_chat_list<'a>(
    chats: &[&'a ChatRecord],
    active_jid: Option<&str>,
    search_query: &str,
    last_snippets: impl Fn(&str) -> Option<&'a str>,
) -> Div {
    let mut list = div()
        .flex()
        .flex_col()
        .w_80()
        .h_full()
        .bg(ThemeColors::base())
        .border_r_1()
        .border_color(rgb(ThemeColors::BORDER))
        // Search Header
        .child(
            div()
                .flex()
                .items_center()
                .h_12()
                .px_3()
                .bg(ThemeColors::mantle())
                .border_b_1()
                .border_color(rgb(ThemeColors::BORDER))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .w_full()
                        .h_8()
                        .px_2()
                        .rounded_md()
                        .bg(rgb(ThemeColors::SURFACE0))
                        .text_size(px(13.0))
                        .text_color(if search_query.is_empty() {
                            ThemeColors::text_muted()
                        } else {
                            ThemeColors::text_primary()
                        })
                        .child(if search_query.is_empty() {
                            "Search chats (Ctrl+K)...".to_string()
                        } else {
                            search_query.to_string()
                        }),
                ),
        );

    // Chat Items or Empty State
    if chats.is_empty() {
        let empty_msg = if search_query.is_empty() {
            "No conversations yet"
        } else {
            "No chats match search"
        };

        list = list.child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.0))
                .text_color(ThemeColors::text_muted())
                .child(empty_msg),
        );
    } else {
        let mut scroll_area = div().flex_1().flex().flex_col().overflow_hidden();

        for chat in chats {
            let is_selected = active_jid.map(|j| j == chat.jid).unwrap_or(false);
            let snippet = last_snippets(&chat.jid);
            let props = ChatItemProps::from_record(chat, snippet, is_selected);
            scroll_area = scroll_area.child(render_chat_item(&props));
        }

        list = list.child(scroll_area);
    }

    list
}
