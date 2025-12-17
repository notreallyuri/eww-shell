use ini::Ini;
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};

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
    let args: Vec<String> = env::args().collect();
    let search_query = args.get(1).map(|s| s.to_lowercase());

    let mut apps = Vec::new();
    let app_dirs = get_application_dirs();

    for dir in app_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
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
        "prism launcher",
        "steam",
        "zen browser",
        "obs studio",
        "cider",
        "discord",
    ];

    let favorites: Vec<App> = apps
        .iter()
        .filter(|app| {
            let name_lower = app.name.to_lowercase();
            fav_keywords.iter().any(|&k| name_lower.contains(k))
        })
        .cloned()
        .collect();

    let filtered_apps = if let Some(query) = &search_query {
        apps.iter()
            .filter(|app| app.name.to_lowercase().contains(query))
            .cloned()
            .collect()
    } else {
        apps.clone()
    };

    let filtered_favorites = if let Some(query) = &search_query {
        favorites
            .iter()
            .filter(|app| app.name.to_lowercase().contains(query))
            .cloned()
            .collect()
    } else {
        favorites.clone()
    };

    let state = AppState {
        apps: filtered_apps,
        favorites: filtered_favorites,
    };

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

    let icon = resolve_icon_path(raw_icon);
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

fn resolve_icon_path(raw_icon: &str) -> String {
    if raw_icon.is_empty() {
        return get_fallback_icon();
    }

    if raw_icon.starts_with('/') {
        if std::path::Path::new(raw_icon).exists() {
            return raw_icon.to_string();
        }
        let stem = std::path::Path::new(raw_icon)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(raw_icon);
        return resolve_icon_path(stem);
    }

    let mut best_match: Option<String> = None;
    let mut best_score = 0;

    for icon in linicon::lookup_icon(raw_icon)
        .filter_map(|r| r.ok())
        .take(15)
    {
        let path = icon.path.to_string_lossy().to_string();
        let score = score_path(&path);

        if score > best_score {
            best_score = score;
            best_match = Some(path);
        }

        if best_score >= 5 {
            break;
        }
    }

    if let Some(path) = best_match {
        return path;
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/yuri".to_string());

    let high_res_candidates = [
        format!("/usr/share/icons/hicolor/scalable/apps/{}.svg", raw_icon),
        format!(
            "{}/.local/share/icons/hicolor/scalable/apps/{}.svg",
            home, raw_icon
        ),
        format!("/usr/share/icons/hicolor/512x512/apps/{}.png", raw_icon),
        format!(
            "{}/.local/share/icons/hicolor/512x512/apps/{}.png",
            home, raw_icon
        ),
        format!("/usr/share/icons/hicolor/256x256/apps/{}.png", raw_icon),
        format!(
            "{}/.local/share/icons/hicolor/256x256/apps/{}.png",
            home, raw_icon
        ),
        format!("/usr/share/icons/hicolor/128x128/apps/{}.png", raw_icon),
        format!("/usr/share/icons/hicolor/48x48/apps/{}.png", raw_icon),
        format!("/usr/share/pixmaps/{}.svg", raw_icon),
        format!("/usr/share/pixmaps/{}.png", raw_icon),
    ];

    for path in high_res_candidates {
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }

    get_fallback_icon()
}

fn get_fallback_icon() -> String {
    let candidates = [
        "/usr/share/icons/Adwaita/symbolic/action/action-unavailable-symbolic.svg",
        "/usr/share/icons/hicolor/48x48/apps/application-default-icon.png",
        "/usr/share/pixmaps/python.png",
    ];
    for path in candidates {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    "".to_string()
}

fn score_path(path: &str) -> i32 {
    if path.ends_with(".svg") || path.contains("/scalable/") {
        return 5;
    }
    if path.contains("512x512") {
        return 4;
    }
    if path.contains("256x256") {
        return 3;
    }
    if path.contains("128x128") {
        return 2;
    }
    if path.contains("64x64") {
        return 1;
    }
    0
}
