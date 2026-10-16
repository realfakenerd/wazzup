use gpui::{
    prelude::*, App, Application, Bounds, WindowBounds, WindowOptions, px, size,
};
use wazzup::ipc::WacliLoader;
use wazzup::ui::MainWindow;

fn main() {
    let state = WacliLoader::load_initial_state();

    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);
        let app_state = state.clone();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| cx.new(|_| MainWindow::with_state(app_state)),
        )
        .expect("failed to open window");

        cx.activate(true);
    });
}
