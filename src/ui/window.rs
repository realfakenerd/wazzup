use gpui::*;
use crate::theme::ThemeColors;

pub struct MainWindow {
    title: String,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            title: "Wazzup".to_string(),
        }
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(ThemeColors::base())
            .text_color(ThemeColors::text_primary())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h_10()
                    .px_4()
                    .bg(ThemeColors::mantle())
                    .border_b_1()
                    .border_color(rgb(ThemeColors::BORDER))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w_3()
                                    .h_3()
                                    .rounded_full()
                                    .bg(ThemeColors::emerald()),
                            )
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(ThemeColors::text_muted())
                            .child("Wayland • GPUI"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("Wazzup — Ready to connect"),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_window_creation() {
        let window = MainWindow::new();
        assert_eq!(window.title, "Wazzup");
    }
}
