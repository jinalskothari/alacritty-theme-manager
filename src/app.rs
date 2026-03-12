use std::{path::PathBuf, process::Command};

use ratatui::widgets::ListState;
use toml_edit::DocumentMut;

use crate::config::{
    apply_theme, current_theme_stem, load_doc, load_themes, remove_theme,
    resolve_config_path, resolve_themes_dir,
};

pub enum ExitAction {
    KeepCurrent,
    RestoreOriginal,
}

pub struct App {
    pub themes: Vec<String>,
    pub list_state: ListState,
    pub original_theme: Option<String>,
    pub is_git_repo: bool,
    pub status_msg: Option<String>,
    config_path: PathBuf,
    themes_dir: PathBuf,
    doc: DocumentMut,
}

impl App {
    pub fn new() -> Self {
        let config_path = resolve_config_path();
        let themes_dir = resolve_themes_dir();
        let doc = load_doc(&config_path);
        let original_theme = current_theme_stem(&doc, &themes_dir);
        let themes = load_themes(&themes_dir);

        let selected = original_theme
            .as_deref()
            .and_then(|o| themes.iter().position(|t| t == o))
            .unwrap_or(0);

        let mut list_state = ListState::default();
        if !themes.is_empty() {
            list_state.select(Some(selected));
        }

        // The git repo is the parent of the themes dir (e.g. ~/.config/alacritty/themes)
        let is_git_repo = themes_dir
            .parent()
            .map(|p| p.join(".git").exists())
            .unwrap_or(false);

        App {
            themes,
            list_state,
            original_theme,
            is_git_repo,
            status_msg: None,
            config_path,
            themes_dir,
            doc,
        }
    }

    pub fn selected_theme(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|i| self.themes.get(i))
            .map(|s| s.as_str())
    }

    pub fn apply_selected(&mut self) {
        if let Some(theme) = self.selected_theme().map(|s| s.to_string()) {
            apply_theme(&mut self.doc, &self.config_path, &self.themes_dir, &theme);
        }
    }

    pub fn move_up(&mut self) {
        if self.themes.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        let next = if i == 0 { self.themes.len() - 1 } else { i - 1 };
        self.list_state.select(Some(next));
        self.status_msg = None;
        self.apply_selected();
    }

    pub fn move_down(&mut self) {
        if self.themes.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        let next = (i + 1) % self.themes.len();
        self.list_state.select(Some(next));
        self.status_msg = None;
        self.apply_selected();
    }

    pub fn restore_original(&mut self) {
        match &self.original_theme.clone() {
            Some(t) => apply_theme(&mut self.doc, &self.config_path, &self.themes_dir, t),
            None => remove_theme(&mut self.doc, &self.config_path, &self.themes_dir),
        }
    }

    /// Run `git pull` in the themes repo, refresh the theme list, and return
    /// a short status string for display in the footer.
    pub fn git_pull(&mut self) -> String {
        let repo_dir = match self.themes_dir.parent() {
            Some(p) => p.to_path_buf(),
            None => return "error: cannot determine repo directory".to_string(),
        };

        let output = Command::new("git")
            .args(["-C", &repo_dir.to_string_lossy(), "pull"])
            .output();

        match output {
            Err(e) => format!("failed to run git: {e}"),
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if !out.status.success() {
                    let msg = stderr.lines().next().unwrap_or("unknown error");
                    return format!("git error: {msg}");
                }

                if stdout.contains("Already up to date") {
                    "Already up to date".to_string()
                } else {
                    // New themes may have been added — reload the list
                    let current = self.selected_theme().map(|s| s.to_string());
                    self.themes = load_themes(&self.themes_dir);

                    // Restore selection to the same theme if still present
                    let idx = current
                        .as_deref()
                        .and_then(|t| self.themes.iter().position(|s| s == t))
                        .unwrap_or(0);
                    self.list_state.select(Some(idx));

                    "Updated — theme list refreshed".to_string()
                }
            }
        }
    }
}
