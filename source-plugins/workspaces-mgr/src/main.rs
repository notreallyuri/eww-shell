use std::collections::HashMap;

use hyprland::data::{Monitors, Workspaces};
use hyprland::event_listener::EventListener;
use hyprland::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct UIWorkspace {
    id: i32,
    occupied: bool,
    active: bool,
}

#[derive(Serialize)]
struct MonitorState {
    items: Vec<UIWorkspace>,
    active_index: i32,
}

#[tokio::main]
async fn main() -> hyprland::Result<()> {
    print_workspace().await;

    let mut event_listener = EventListener::new();

    event_listener.add_workspace_changed_handler(|_| {
        tokio::spawn(async {
            print_workspace().await;
        });
    });

    event_listener.add_workspace_added_handler(|_| {
        tokio::spawn(async {
            print_workspace().await;
        });
    });

    event_listener.add_workspace_deleted_handler(|_| {
        tokio::spawn(async {
            print_workspace().await;
        });
    });

    event_listener.add_window_opened_handler(|_| {
        tokio::spawn(async {
            print_workspace().await;
        });
    });

    event_listener.add_window_closed_handler(|_| {
        tokio::spawn(async {
            print_workspace().await;
        });
    });

    event_listener.add_window_moved_handler(|_| {
        tokio::spawn(async {
            print_workspace().await;
        });
    });

    event_listener.start_listener_async().await
}

async fn print_workspace() {
    let monitors_res = Monitors::get_async().await;
    let workspaces_res = Workspaces::get_async().await;

    if let (Ok(monitors), Ok(workspaces)) = (monitors_res, workspaces_res) {
        let mut output: HashMap<String, MonitorState> = HashMap::new();

        for monitor in monitors {
            let active_id = monitor.active_workspace.id;
            let chunk_offset = ((active_id - 1) / 5) * 5;

            let mut ui_items = Vec::new();
            let mut active_index = -1;

            for i in 1..=5 {
                let id = chunk_offset + i;
                let occupied = workspaces.iter().any(|w| w.id == id);
                let is_active = id == active_id;

                if is_active {
                    active_index = i - 1;
                }

                ui_items.push(UIWorkspace {
                    id,
                    occupied,
                    active: is_active,
                });
            }

            output.insert(
                monitor.id.to_string(),
                MonitorState {
                    items: ui_items,
                    active_index,
                },
            );
        }

        if let Ok(json) = serde_json::to_string(&output) {
            println!("{}", json);
        }
    }
}
