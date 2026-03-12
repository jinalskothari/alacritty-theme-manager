mod app;
mod config;
mod ui;

use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, ExitAction};
use config::resolve_themes_dir;

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<ExitAction> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Enter => return Ok(ExitAction::KeepCurrent),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(ExitAction::RestoreOriginal),
                KeyCode::Char('u') if app.is_git_repo => {
                    app.status_msg = Some("Updating themes…".to_string());
                    terminal.draw(|f| ui::draw(f, app))?;
                    let msg = app.git_pull();
                    app.status_msg = Some(msg);
                }
                _ => {}
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut app = App::new();

    if app.themes.is_empty() {
        eprintln!(
            "No themes found in {}.\n\nInstall themes first:\n  git clone https://github.com/alacritty/alacritty-theme ~/.config/alacritty/themes",
            resolve_themes_dir().display()
        );
        return Ok(());
    }

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
