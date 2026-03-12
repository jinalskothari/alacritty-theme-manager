# Alacritty Theme Manager

A fast terminal TUI for live-previewing and switching [Alacritty](https://github.com/alacritty/alacritty) themes.

**`atm`** runs inside Alacritty itself — as you scroll through themes, the terminal hot-reloads instantly and the TUI's own colors update in real time, giving you a true live preview.

## Features

- **Live preview** — navigate with `↑↓` or `jk`; the theme applies immediately
- **Full color demo** — built-in preview panel shows all 16 ANSI colors via palette swatches, syntax-highlighted code, and terminal output
- **Safe config editing** — uses `toml_edit` for lossless TOML updates; only the theme import entry is touched, all other config is preserved
- **Git integration** — press `u` to `git pull` the themes repo and refresh the list
- **Flexible paths** — auto-detects config and themes dir; override with `ATM_CONFIG` / `ATM_THEMES_DIR`

## Demo

![Alacritty Theme Manager screenshot](assets/Alacritty%20Theme%20Manager%20Screenshot.png)

```
┌─ Themes ────────────┐ ┌──── Preview ────────────────────────────────┐
│ > catppuccin_mocha  │ │  ── palette ──────────────────────────────── │
│   dracula           │ │  ██ black      ██ bright black               │
│   gruvbox_dark      │ │  ██ red        ██ bright red                 │
│   nord              │ │  ...                                         │
│   one_dark          │ │  ── code ─────────────────────────────────── │
│   rose_pine         │ │  struct Server { // runtime config           │
│   tokyo_night       │ │  async fn connect(srv: Server) -> Result ... │
│   ...               │ │  ── terminal ─────────────────────────────── │
│                     │ │  ~/projects (main) $ cargo build             │
│                     │ │       Compiling server v0.1.0                │
└─────────────────────┘ └──────────────────────────────────────────────┘
┌─ ↑↓ jk  navigate    Enter  keep    Esc/q  restore & exit    u  update ┐
└────────────────────────────────────────────────────────────────────────┘
```

## Installation

### Prerequisites

Install the [alacritty-theme](https://github.com/alacritty/alacritty-theme) collection:

```sh
mkdir -p ~/.config/alacritty
git clone https://github.com/alacritty/alacritty-theme ~/.config/alacritty/themes
```

### From source

```sh
git clone https://github.com/jinalskothari/alacritty-theme-manager
cd alacritty-theme-manager
cargo install --path .
```

## Usage

```sh
atm
```

| Key | Action |
|-----|--------|
| `↑` / `k` | Previous theme |
| `↓` / `j` | Next theme |
| `Enter` | Keep selected theme and exit |
| `Esc` / `q` | Restore original theme and exit |
| `u` | `git pull` themes repo and refresh list *(only shown when repo detected)* |

## Configuration

Paths are resolved in this order: **env var → `$XDG_CONFIG_HOME` → `~/.config`** (macOS also checks `~/Library/Application Support`).

| Variable | Default | Description |
|----------|---------|-------------|
| `ATM_CONFIG` | `~/.config/alacritty/alacritty.toml` | Path to your Alacritty config |
| `ATM_THEMES_DIR` | `~/.config/alacritty/themes/themes/` | Directory containing theme `.toml` files |

Example:

```sh
ATM_THEMES_DIR=~/my-themes atm
```

## How it works

`atm` edits the `[general].import` array in your `alacritty.toml` as you navigate:

```toml
[general]
import = [
    "~/.config/alacritty/themes/themes/catppuccin_mocha.toml"
]
```

Alacritty watches the config file and hot-reloads automatically. Only the theme entry is modified — all other settings are left untouched.

On exit:
- **`Enter`** — the last selected theme stays in your config
- **`Esc`/`q`** — the original theme (or no theme if none was set) is restored

## Requirements

- [Alacritty](https://github.com/alacritty/alacritty) with hot-reload enabled (default)
- Rust 1.85+ (edition 2024)

## License

MIT

---

*Built entirely with [Claude Code](https://claude.ai/code) inside [Zed](https://zed.dev).*
