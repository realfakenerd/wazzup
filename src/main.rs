use std::sync::Arc;
use gpui::{
    prelude::*, App, Application, Bounds, WindowBounds, WindowOptions, px, size,
};
use tokio::sync::mpsc;
use wazzup::ipc::{ChannelWhatsAppClient, WacliActor, WacliLoader};
use wazzup::ui::MainWindow;

fn main() {
    let state = WacliLoader::load_initial_state();

    let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
    let (tx, rx) = mpsc::channel(32);
    let _actor_handle = rt.spawn(WacliActor::new(rx).run());
    let client = Arc::new(ChannelWhatsAppClient::new(tx));
    // Keep runtime alive for app process lifetime
    let _rt = Box::leak(Box::new(rt));

    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);
        let app_state = state.clone();
        let app_client = client.clone();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                let focus_handle = cx.focus_handle();
                focus_handle.focus(window);
                cx.new(|_| {
                    let mut mw = MainWindow::with_client(app_state, app_client);
                    mw.focus_handle = Some(focus_handle);
                    mw
                })
            },
        )
        .expect("failed to open window");

        cx.activate(true);
    });
}
