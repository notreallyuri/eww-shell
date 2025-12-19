use hyprland::data::Client;
use hyprland::event_listener::EventListener;
use hyprland::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct WindowState {
    title: String,
    class: String,
}

#[tokio::main]
async fn main() -> hyprland::Result<()> {
    print_window_title().await;

    let mut event_listener = EventListener::new();

    event_listener.add_active_window_changed_handler(|_| {
        tokio::spawn(async {
            print_window_title().await;
        });
    });

    event_listener.add_window_title_changed_handler(|_| {
        tokio::spawn(async {
            print_window_title().await;
        });
    });

    event_listener.start_listener_async().await
}

async fn print_window_title() {
    let state = Client::get_active_async().await.ok().flatten().map_or(
        WindowState {
            title: "No active window".to_string(),
            class: String::new(),
        },
        |window| WindowState {
            title: window.title,
            class: window.class,
        },
    );

    if let Ok(json) = serde_json::to_string(&state) {
        println!("{}", json);
    }
}
