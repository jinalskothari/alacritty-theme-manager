# alacritty-theme-manager (`atm`)

A terminal TUI (Rust + Ratatui) for live-previewing and switching Alacritty themes.

## Key Insight

A Ratatui app running *inside* Alacritty inherits the terminal's color palette. Using only
`Color::Indexed(0..=15)` (never `Color::Rgb`), the TUI itself acts as a live preview —
the moment Alacritty hot-reloads the config, every color in the UI updates instantly.

## How It Works

1. Resolve config and themes dir (env vars → XDG → defaults)
2. Show a scrollable list of theme names (left 30%) + color preview panel (right 70%)
3. As cursor moves → write selected theme into `alacritty.toml` → Alacritty hot-reloads → TUI colors update live
4. `Enter` — keep selected theme and exit
5. `Esc`/`q` — restore original theme and exit
6. `u` — `git pull` the themes repo and refresh the list (only shown when repo detected)

## Path Resolution

Priority: env var → XDG_CONFIG_HOME → `~/.config` → macOS Library (config only)

| Purpose        | Env var          | Default                                        |
|----------------|------------------|------------------------------------------------|
| Alacritty config | `ATM_CONFIG`   | `~/.config/alacritty/alacritty.toml`           |
| Themes dir     | `ATM_THEMES_DIR` | `~/.config/alacritty/themes/themes/`           |

## Config Update Strategy

Use `toml_edit` for lossless TOML editing (preserves comments and formatting).
Only the theme entry inside `[general].import` is touched — all other config is untouched.
Theme import path detection uses `~`-expansion and parent-dir comparison, not substring matching,
so custom paths work correctly.

## TUI Color Rule

**Never use `Color::Rgb(...)` in UI widgets.** Use only `Color::Indexed(0..=15)`.
This ensures the UI demonstrates the active theme rather than overriding it.

## Module Structure

```
src/
  main.rs    — terminal setup, event loop, entry point
  app.rs     — App struct, navigation, theme apply, git pull
  config.rs  — path resolution, TOML read/write, theme list loading
  ui.rs      — draw(), preview_lines() with all 16-color demo content
```

## Preview Panel Sections

1. **Palette** — all 16 ANSI colors as normal+bright pairs
2. **Code** — fake Rust snippet using syntax colors (keywords, types, strings, comments, macros)
3. **Terminal** — fake cargo output using status colors (success, warning, error)

## Layout

```
┌─ Themes (30%) ──────┐ ┌──── Preview (70%) ──────────────────────────┐
│ > catppuccin_mocha  │ │  ── palette ──────────────────────────────── │
│   dracula           │ │  ██ black      ██ bright black               │
│   gruvbox_dark      │ │  ...                                         │
│   nord              │ │  ── code ─────────────────────────────────── │
│   one_dark          │ │  struct Server { // comment                  │
│   ...               │ │  ...                                         │
│                     │ │  ── terminal ─────────────────────────────── │
│                     │ │  ~/projects (main) $ cargo build             │
└─────────────────────┘ └──────────────────────────────────────────────┘
┌─ ↑↓ jk navigate   Enter keep   Esc/q restore & exit   u update ─────┐
└──────────────────────────────────────────────────────────────────────┘
```

## Crates

- `ratatui` — TUI framework
- `crossterm` — terminal backend
- `toml_edit` — lossless TOML parsing/editing
- `dirs` — home directory resolution
