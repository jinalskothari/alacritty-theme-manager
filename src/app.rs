use std::path::PathBuf;

use ratatui::widgets::ListState;
use toml_edit::DocumentMut;

use crate::config::{
    apply_theme, config_path, current_theme_stem, load_doc, load_themes, remove_theme,
};

pub enum ExitAction {
    KeepCurrent,
    RestoreOriginal,
}

pub struct App {
    pub themes: Vec<String>,
    pub list_state: ListState,
    pub original_theme: Option<String>,
    config_path: PathBuf,
    doc: DocumentMut,
}

impl App {
    pub fn new() -> Self {
        let config_path = config_path();
        let doc = load_doc(&config_path);
        let original_theme = current_theme_stem(&doc);
        let themes = load_themes();

        let selected = original_theme
            .as_deref()
            .and_then(|o| themes.iter().position(|t| t == o))
            .unwrap_or(0);

        let mut list_state = ListState::default();
        if !themes.is_empty() {
            list_state.select(Some(selected));
        }

        App { themes, list_state, original_theme, config_path, doc }
    }

    pub fn selected_theme(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|i| self.themes.get(i))
            .map(|s| s.as_str())
    }

    pub fn apply_selected(&mut self) {
        if let Some(theme) = self.selected_theme().map(|s| s.to_string()) {
            apply_theme(&mut self.doc, &self.config_path, &theme);
        }
    }

    pub fn move_up(&mut self) {
        if self.themes.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        let next = if i == 0 { self.themes.len() - 1 } else { i - 1 };
        self.list_state.select(Some(next));
        self.apply_selected();
    }

    pub fn move_down(&mut self) {
        if self.themes.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        let next = (i + 1) % self.themes.len();
        self.list_state.select(Some(next));
        self.apply_selected();
    }

    pub fn restore_original(&mut self) {
        match &self.original_theme.clone() {
            Some(t) => apply_theme(&mut self.doc, &self.config_path, t),
            None => remove_theme(&mut self.doc, &self.config_path),
        }
    }
}
