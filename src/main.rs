use gpui::{
    prelude::*, App, Application, Bounds, WindowBounds, WindowOptions, px, size,
};
use wazzup::ui::MainWindow;

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| MainWindow::new()),
        )
        .expect("failed to open window");

        cx.activate(true);
    });
}
