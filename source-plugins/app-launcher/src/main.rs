use ini::Ini;
use linicon::lookup_icon;
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Serialize, Clone)]
struct App {
    name: String,
    icon: String,
    exec: String,
    terminal: bool,
}

#[derive(Serialize)]
struct AppState {
    apps: Vec<App>,
    favorites: Vec<App>,
}

impl AppState {}

fn main() {
    let mut apps = Vec::new();
    let app_dirs = get_application_dirs();

    for dir in app_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("desktop")
                && let Some(app) = parse_desktop_file(path)
            {
                apps.push(app);
            }
        }
    }

    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);

    let fav_keywords = [
        "neovim",
        "nvim",
        "prism launcher",
        "steam",
        "zen browser",
        "zen",
        "obs studio",
        "cider",
        "discord",
    ];

    let favorites: Vec<App> = apps
        .iter()
        .filter(|app| {
            let name_lower = app.name.to_lowercase();
            // Check if the app name contains any of our keywords
            fav_keywords.iter().any(|&k| name_lower.contains(k))
        })
        .cloned()
        .collect();

    let state = AppState { apps, favorites };

    println!("{}", serde_json::to_string(&state).unwrap());
}

fn get_application_dirs() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(user_share) = dirs::data_local_dir() {
        paths.push(user_share.join("applications"));
    }

    let xdg_data_dirs =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

    for dir in xdg_data_dirs.split(':') {
        paths.push(PathBuf::from(dir).join("applications"));
    }

    paths
}

fn parse_desktop_file(path: &Path) -> Option<App> {
    let conf = Ini::load_from_file(path).ok()?;
    let section = conf.section(Some("Desktop Entry"))?;

    if section.get("NoDisplay") == Some("true") {
        return None;
    }

    let name = section.get("Name")?.to_string();
    let raw_icon = section.get("Icon").unwrap_or("");

    let icon = if raw_icon.is_empty() {
        "dialog-question".to_string()
    } else if raw_icon.starts_with('/') {
        raw_icon.to_string()
    } else {
        lookup_icon(raw_icon)
            .next() // Get first match: Option<Result<...>>
            .and_then(|r| r.ok()) // Convert Result to Option (discard error)
            .map(|i| i.path.to_string_lossy().to_string()) // Get the path
            .unwrap_or_else(|| raw_icon.to_string())
    };

    let raw_exec = section.get("Exec")?;

    let exec = raw_exec
        .replace("%f", "")
        .replace("%F", "")
        .replace("%u", "")
        .replace("%U", "")
        .trim()
        .to_string();

    let terminal = section.get("Terminal").unwrap_or("false") == "true";

    Some(App {
        name,
        icon,
        exec,
        terminal,
    })
}
