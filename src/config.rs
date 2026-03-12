use std::{fs, path::PathBuf};

use toml_edit::{Array, DocumentMut, Value};

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("cannot find home directory")
        .join(".config/alacritty/alacritty.toml")
}

pub fn themes_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot find home directory")
        .join(".config/alacritty/themes/themes")
}

pub fn load_doc(path: &PathBuf) -> DocumentMut {
    match fs::read_to_string(path) {
        Ok(s) => s.parse().unwrap_or_default(),
        Err(_) => DocumentMut::new(),
    }
}

fn find_theme_index(doc: &DocumentMut) -> Option<usize> {
    let imports = doc
        .get("general")
        .and_then(|g| g.get("import"))
        .and_then(|i| i.as_array())?;
    imports.iter().position(|v| {
        v.as_str()
            .map(|s| s.contains("/themes/themes/"))
            .unwrap_or(false)
    })
}

pub fn current_theme_stem(doc: &DocumentMut) -> Option<String> {
    let idx = find_theme_index(doc)?;
    let imports = doc
        .get("general")
        .and_then(|g| g.get("import"))
        .and_then(|i| i.as_array())?;
    let path = imports.get(idx)?.as_str()?;
    PathBuf::from(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

pub fn apply_theme(doc: &mut DocumentMut, path: &PathBuf, theme_stem: &str) {
    let theme_path = format!("~/.config/alacritty/themes/themes/{}.toml", theme_stem);

    if doc.get("general").is_none() {
        doc["general"] = toml_edit::table();
    }
    let general = doc["general"].as_table_mut().expect("general is a table");

    if let Some(import) = general.get_mut("import").and_then(|i| i.as_array_mut()) {
        let idx = import
            .iter()
            .position(|v| v.as_str().map(|s| s.contains("/themes/themes/")).unwrap_or(false));
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

    let _ = fs::write(path, doc.to_string());
}

pub fn remove_theme(doc: &mut DocumentMut, path: &PathBuf) {
    if let Some(general) = doc.get_mut("general").and_then(|g| g.as_table_mut()) {
        if let Some(import) = general.get_mut("import").and_then(|i| i.as_array_mut()) {
            let idx = import
                .iter()
                .position(|v| v.as_str().map(|s| s.contains("/themes/themes/")).unwrap_or(false));
            if let Some(idx) = idx {
                import.remove(idx);
            }
        }
    }
    let _ = fs::write(path, doc.to_string());
}

pub fn load_themes() -> Vec<String> {
    let Ok(entries) = fs::read_dir(themes_dir()) else {
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
