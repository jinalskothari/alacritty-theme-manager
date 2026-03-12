# Alacritty Theme Changer

A terminal TUI (Rust + Ratatui) for live-previewing and switching Alacritty themes.

## Key Insight

A Ratatui app running *inside* Alacritty inherits the terminal's color palette. If we use only standard ANSI colors (Color::Reset, Color::Indexed 0–15) rather than hardcoded RGB, the TUI itself acts as a live preview — the moment Alacritty hot-reloads the config, every color in the UI updates instantly.

## How It Works

1. Read all `.toml` files from `~/.config/alacritty/themes/themes/`
2. Show a scrollable list of theme names
3. As cursor moves → immediately write the selected theme to `alacritty.toml` import → Alacritty hot-reloads → TUI colors update live
4. Press Enter or `q`/Esc to confirm and exit (keeping current theme), or press Esc to restore original theme on exit

## File Paths

- Alacritty config: `~/.config/alacritty/alacritty.toml`
- Themes directory: `~/.config/alacritty/themes/themes/*.toml`

## Config Update Strategy

Parse the `[general]` section of `alacritty.toml` and update (or insert) the `import` array to point at the selected theme. Preserve all other config untouched.

Use `toml_edit` crate to do lossless TOML editing (preserves comments and formatting).

## Behavior on Exit

- `Enter` / `q` — keep selected theme, exit
- `Esc` — restore original theme (the one active when the TUI launched), exit

## TUI Color Rule

**Never use `Color::Rgb(...)` in UI widgets.** Use only:
- `Color::Reset`
- `Color::Indexed(0..=15)` (the 16 ANSI colors)

This ensures the UI demonstrates the active theme rather than overriding it.

## Crates

- `ratatui` — TUI framework
- `crossterm` — terminal backend
- `toml_edit` — lossless TOML parsing/editing
- `dirs` — resolve `~` to home directory

## Layout

```
┌─ Alacritty Theme Changer ──────────────────┐
│                                             │
│  > catppuccin_mocha                         │
│    dracula                                  │
│    gruvbox_dark                             │
│    nord                                     │
│    one_dark                                 │
│    ...                                      │
│                                             │
│  ↑↓ navigate  Enter/q confirm  Esc restore  │
└─────────────────────────────────────────────┘
```
