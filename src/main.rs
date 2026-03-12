use std::{
    fs,
    io,
    path::PathBuf,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use toml_edit::{Array, DocumentMut, Value};

// ── paths ────────────────────────────────────────────────────────────────────

fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("cannot find home directory")
        .join(".config/alacritty/alacritty.toml")
}

fn themes_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot find home directory")
        .join(".config/alacritty/themes/themes")
}

// ── config helpers ───────────────────────────────────────────────────────────

fn load_doc(path: &PathBuf) -> DocumentMut {
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

fn current_theme_stem(doc: &DocumentMut) -> Option<String> {
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

fn apply_theme(doc: &mut DocumentMut, path: &PathBuf, theme_stem: &str) {
    let theme_path = format!(
        "~/.config/alacritty/themes/themes/{}.toml",
        theme_stem
    );

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

fn remove_theme(doc: &mut DocumentMut, path: &PathBuf) {
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

// ── theme list ───────────────────────────────────────────────────────────────

fn load_themes() -> Vec<String> {
    let dir = themes_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return vec![];
    };
    let mut themes: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("toml") {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    themes.sort();
    themes
}

// ── app state ────────────────────────────────────────────────────────────────

enum ExitAction {
    KeepCurrent,
    RestoreOriginal,
}

struct App {
    themes: Vec<String>,
    list_state: ListState,
    original_theme: Option<String>,
    config_path: PathBuf,
    doc: DocumentMut,
    confirming_exit: bool,
}

impl App {
    fn new() -> Self {
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

        App {
            themes,
            list_state,
            original_theme,
            config_path,
            doc,
            confirming_exit: false,
        }
    }

    fn selected_theme(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|i| self.themes.get(i))
            .map(|s| s.as_str())
    }

    fn apply_selected(&mut self) {
        if let Some(theme) = self.selected_theme().map(|s| s.to_string()) {
            apply_theme(&mut self.doc, &self.config_path, &theme);
        }
    }

    fn move_up(&mut self) {
        if self.themes.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let next = if i == 0 { self.themes.len() - 1 } else { i - 1 };
        self.list_state.select(Some(next));
        self.apply_selected();
    }

    fn move_down(&mut self) {
        if self.themes.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let next = (i + 1) % self.themes.len();
        self.list_state.select(Some(next));
        self.apply_selected();
    }

    fn restore_original(&mut self) {
        match &self.original_theme.clone() {
            Some(t) => apply_theme(&mut self.doc, &self.config_path, t),
            None => remove_theme(&mut self.doc, &self.config_path),
        }
    }
}

// ── UI ───────────────────────────────────────────────────────────────────────

fn draw(f: &mut ratatui::Frame, app: &App) {
    let area = f.area();

    let footer_height = if app.confirming_exit { 4 } else { 3 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(footer_height)])
        .split(area);

    // theme list
    let items: Vec<ListItem> = app
        .themes
        .iter()
        .map(|t| ListItem::new(Line::from(Span::raw(t.clone()))))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Alacritty Theme Changer ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Indexed(4))),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Indexed(0))
                .bg(Color::Indexed(4))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state.clone());

    // footer
    let footer_lines = if app.confirming_exit {
        let current = app.selected_theme().unwrap_or("none");
        let original = app.original_theme.as_deref().unwrap_or("none");
        vec![
            Line::from(vec![
                Span::raw("Keep "),
                Span::styled(
                    format!("\"{}\"", current),
                    Style::default()
                        .fg(Color::Indexed(2))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("?"),
            ]),
            Line::from(vec![
                Span::styled("[y] ", Style::default().fg(Color::Indexed(2))),
                Span::raw("keep   "),
                Span::styled("[n] ", Style::default().fg(Color::Indexed(1))),
                Span::raw("restore "),
                Span::styled(
                    format!("\"{}\"", original),
                    Style::default().fg(Color::Indexed(3)),
                ),
                Span::styled("   [Esc] ", Style::default().fg(Color::Indexed(8))),
                Span::raw("cancel"),
            ]),
        ]
    } else {
        vec![Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Indexed(3))),
            Span::raw("navigate   "),
            Span::styled("Esc/q ", Style::default().fg(Color::Indexed(3))),
            Span::raw("exit"),
        ])]
    };

    let footer = Paragraph::new(footer_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Indexed(8))),
        )
        .alignment(Alignment::Center);

    f.render_widget(footer, chunks[1]);
}

// ── main loop ────────────────────────────────────────────────────────────────

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<ExitAction> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.confirming_exit {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        return Ok(ExitAction::KeepCurrent);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        return Ok(ExitAction::RestoreOriginal);
                    }
                    KeyCode::Esc => {
                        app.confirming_exit = false;
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.confirming_exit = true;
                    }
                    _ => {}
                }
            }
        }
    }
}

// ── entry point ──────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let mut app = App::new();

    if app.themes.is_empty() {
        eprintln!(
            "No themes found in {}.\n\nInstall themes first:\n  git clone https://github.com/alacritty/alacritty-theme ~/.config/alacritty/themes",
            themes_dir().display()
        );
        return Ok(());
    }

    // Apply the currently-highlighted theme on launch for immediate preview
    app.apply_selected();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let action = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match action? {
        ExitAction::KeepCurrent => {}
        ExitAction::RestoreOriginal => app.restore_original(),
    }

    Ok(())
}
