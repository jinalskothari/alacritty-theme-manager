use std::{fs, path::PathBuf};

use toml_edit::{Array, DocumentMut, Value};

// ── path resolution ───────────────────────────────────────────────────────────
//
// Priority: ATC_* env var → XDG_CONFIG_HOME → ~/.config → macOS Library
//
// ATM_CONFIG and ATM_THEMES_DIR are our own app-specific vars, chosen to avoid
// any conflict with variables Alacritty or the system already sets.

pub fn resolve_config_path() -> PathBuf {
    if let Ok(p) = std::env::var("ATM_CONFIG") {
        return PathBuf::from(p);
    }
    let candidates = config_candidates();
    candidates
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| default_config_path())
}

pub fn resolve_themes_dir() -> PathBuf {
    if let Ok(p) = std::env::var("ATM_THEMES_DIR") {
        return PathBuf::from(p);
    }
    let candidates = themes_candidates();
    candidates
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| default_themes_dir())
}

fn home() -> PathBuf {
    dirs::home_dir().expect("cannot find home directory")
}

fn xdg_config_home() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home().join(".config"))
}

fn default_config_path() -> PathBuf {
    xdg_config_home().join("alacritty/alacritty.toml")
}

fn default_themes_dir() -> PathBuf {
    xdg_config_home().join("alacritty/themes/themes")
}

fn config_candidates() -> Vec<PathBuf> {
    let mut v = vec![xdg_config_home().join("alacritty/alacritty.toml")];
    #[cfg(target_os = "macos")]
    v.push(home().join("Library/Application Support/alacritty/alacritty.toml"));
    v
}

fn themes_candidates() -> Vec<PathBuf> {
    vec![xdg_config_home().join("alacritty/themes/themes")]
}

// ── TOML helpers ──────────────────────────────────────────────────────────────

pub fn load_doc(path: &PathBuf) -> DocumentMut {
    match fs::read_to_string(path) {
        Ok(s) => s.parse().unwrap_or_default(),
        Err(_) => DocumentMut::new(),
    }
}

/// Build the path string written into alacritty.toml for a given theme.
/// Uses `~/…` when the themes dir is under the home directory.
fn make_theme_path(themes_dir: &PathBuf, stem: &str) -> String {
    let full = themes_dir.join(format!("{}.toml", stem));
    if let Ok(rel) = full.strip_prefix(home()) {
        return format!("~/{}", rel.to_string_lossy());
    }
    full.to_string_lossy().into_owned()
}

/// Returns true if an import entry path refers to a file inside `themes_dir`.
fn is_theme_import(entry: &str, themes_dir: &PathBuf) -> bool {
    // Expand a leading `~` for comparison purposes only.
    let expanded = if entry.starts_with("~/") {
        home().join(&entry[2..])
    } else {
        PathBuf::from(entry)
    };
    expanded.parent().map(|p| p == themes_dir).unwrap_or(false)
}

fn find_theme_index(doc: &DocumentMut, themes_dir: &PathBuf) -> Option<usize> {
    let imports = doc
        .get("general")
        .and_then(|g| g.get("import"))
        .and_then(|i| i.as_array())?;
    imports
        .iter()
        .position(|v| v.as_str().map(|s| is_theme_import(s, themes_dir)).unwrap_or(false))
}

pub fn current_theme_stem(doc: &DocumentMut, themes_dir: &PathBuf) -> Option<String> {
    let idx = find_theme_index(doc, themes_dir)?;
    let imports = doc
        .get("general")
        .and_then(|g| g.get("import"))
        .and_then(|i| i.as_array())?;
    let path_str = imports.get(idx)?.as_str()?;
    PathBuf::from(path_str)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

pub fn apply_theme(
    doc: &mut DocumentMut,
    config_path: &PathBuf,
    themes_dir: &PathBuf,
    stem: &str,
) {
    let theme_path = make_theme_path(themes_dir, stem);

    if doc.get("general").is_none() {
        doc["general"] = toml_edit::table();
    }
    let general = doc["general"].as_table_mut().expect("general is a table");

    if let Some(import) = general.get_mut("import").and_then(|i| i.as_array_mut()) {
        let idx = import
            .iter()
            .position(|v| v.as_str().map(|s| is_theme_import(s, themes_dir)).unwrap_or(false));
        if let Some(idx) = idx {
            import.replace(idx, Value::from(theme_path));
        } else {
            import.push(theme_path);
        }
    } else {
        let mut arr = Array::new();
        arr.push(theme_path);
        general["import"] = toml_edit::value(arr);
    }

    let _ = fs::write(config_path, doc.to_string());
}

pub fn remove_theme(doc: &mut DocumentMut, config_path: &PathBuf, themes_dir: &PathBuf) {
    if let Some(general) = doc.get_mut("general").and_then(|g| g.as_table_mut()) {
        if let Some(import) = general.get_mut("import").and_then(|i| i.as_array_mut()) {
            let idx = import
                .iter()
                .position(|v| v.as_str().map(|s| is_theme_import(s, themes_dir)).unwrap_or(false));
            if let Some(idx) = idx {
                import.remove(idx);
            }
        }
    }
    let _ = fs::write(config_path, doc.to_string());
}

// ── theme list ────────────────────────────────────────────────────────────────

pub fn load_themes(dir: &PathBuf) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return vec![];
    };
    let mut themes: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("toml") {
                p.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    themes.sort();
    themes
}
