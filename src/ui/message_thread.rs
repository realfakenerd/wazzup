use gpui::{div, px, rgb, Div, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled};
use crate::db::repository::{ChatRecord, MessageRecord};
use crate::theme::ThemeColors;
use crate::ui::chat_item::extract_initials;
use crate::ui::message_bubble::{render_message_bubble, MessageBubbleProps};

pub fn render_message_thread(
    active_chat: Option<&ChatRecord>,
    messages: &[MessageRecord],
) -> Div {
    match active_chat {
        None => div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(ThemeColors::base())
            .gap_3()
            .child(
                div()
                    .w_16()
                    .h_16()
                    .rounded_full()
                    .bg(rgb(ThemeColors::SURFACE0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(28.0))
                    .child("💬"),
            )
            .child(
                div()
                    .text_size(px(18.0))
                    .text_color(rgb(ThemeColors::TEXT_PRIMARY))
                    .child("Wazzup Desktop"),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(ThemeColors::text_muted())
                    .child("Select a conversation from the sidebar to start chatting"),
            ),
        Some(chat) => {
            let chat_name = chat.name.clone().unwrap_or_else(|| chat.jid.clone());
            let initials = extract_initials(&chat_name);

            div()
                .flex_1()
                .flex()
                .flex_col()
                .h_full()
                .bg(rgb(0x0c1317)) // WhatsApp Dark Background
                // Header Bar
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .h(px(56.0))
                        .px_4()
                        .bg(ThemeColors::mantle())
                        .border_b_1()
                        .border_color(rgb(ThemeColors::BORDER))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .w_10()
                                        .h_10()
                                        .rounded_full()
                                        .bg(rgb(ThemeColors::SURFACE1))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(px(14.0))
                                        .text_color(rgb(ThemeColors::TEXT_PRIMARY))
                                        .child(initials),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_size(px(15.0))
                                                .text_color(rgb(ThemeColors::TEXT_PRIMARY))
                                                .child(chat_name),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .text_color(ThemeColors::text_muted())
                                                .child(chat.jid.clone()),
                                        ),
                                ),
                        ),
                )
                // Message Scrollback
                .child({
                    let mut msg_list = div()
                        .id("message_thread_scroll")
                        .flex_1()
                        .flex()
                        .flex_col()
                        .overflow_y_scroll()
                        .py_3();

                    if messages.is_empty() {
                        msg_list = msg_list.child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(13.0))
                                .text_color(ThemeColors::text_muted())
                                .child("No messages in this chat yet"),
                        );
                    } else {
                        for msg in messages {
                            let props = MessageBubbleProps::from_record(msg);
                            msg_list = msg_list.child(render_message_bubble(&props));
                        }
                    }

                    msg_list
                })
        }
    }
}
