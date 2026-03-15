# Known Issues & Improvement Areas

Tracked issues found during code review. Ordered by severity within each category.

---

## Bugs

### ~~B1 — List scroll offset lost on render (`ui.rs:48`)~~ — **Fixed**

`draw()` now takes `&mut App`; the `list_state` clone is gone. The offset is computed
before each render to keep the selected item centred in the visible area, pinning to
the top/bottom only near the ends of the list.

---

### B2 — Config write errors are silently discarded (`config.rs:144, 158`) — **High**

Both `apply_theme` and `remove_theme` end with:

```rust
let _ = fs::write(config_path, doc.to_string());
```

If the write fails (read-only file, disk full, bad permissions), the theme silently does
not change and the user receives no feedback.

**Fix:** Return a `Result` from both functions and surface errors in the footer status bar.

---

### B3 — No panic cleanup guard — terminal left broken on crash (`main.rs:64`) — **Medium**

Terminal cleanup (`disable_raw_mode`, `LeaveAlternateScreen`) is called imperatively in
`main`. A panic anywhere inside `run_app` bypasses this code entirely, leaving the user's
terminal in raw mode with no visible cursor.

**Fix:** Use a RAII cleanup guard (via `Drop`) or wrap the event loop in
`std::panic::catch_unwind`.

---

### B4 — Config modified before TUI starts (`main.rs:56`) — **Medium**

`app.apply_selected()` is called before `enable_raw_mode()`. This writes the first
(original) theme to disk as a side effect at startup. If the binary is killed between
this call and the event loop, the config has been written once unnecessarily. More
importantly, combined with B3, a panic at startup permanently applies whatever theme was
selected at index 0 if no original theme was detected.

**Fix:** Defer the first `apply_selected()` call until the user actually moves the
cursor (i.e., trigger it only inside `move_up` / `move_down`), leaving the config
untouched if the user opens and immediately closes `atm`.

---

## Code Quality

### Q1 — Comment typo (`config.rs:6`) — **Low**

```rust
// Priority: ATC_* env var → …
```

Should read `ATM_*` (the actual env var prefix used throughout the code).

---

### Q2 — `&PathBuf` in function signatures should be `&Path` — **Low**

All public and private config functions (`load_doc`, `apply_theme`, `remove_theme`,
`load_themes`, `make_theme_path`, `is_theme_import`, `find_theme_index`,
`current_theme_stem`) accept `&PathBuf`. The idiomatic Rust type is `&Path`, which
accepts `Path`, `PathBuf`, and string literals without forcing callers to take a reference
to an owned buffer. See [Rust API guidelines C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html).

---

### Q3 — `home()` panics instead of propagating an error (`config.rs:35`) — **Medium**

```rust
fn home() -> PathBuf {
    dirs::home_dir().expect("cannot find home directory")
}
```

A missing home directory panics the process rather than printing a useful error and
exiting cleanly. `main` is the right place to handle this.

---

### Q4 — `current_theme_stem` traverses the import array twice (`config.rs:103`) — **Low**

`current_theme_stem` calls `find_theme_index` (iterates the imports array) and then
re-fetches the same array to index into it. Both lookups can be merged into a single pass.

---

## Architecture

### A1 — `App::new()` bundles path resolution, I/O, and state init — **Low**

`App::new()` resolves paths, reads and parses the TOML doc, detects the original theme,
loads themes, and checks for a `.git` directory — all in one infallible constructor. This
makes it impossible to return errors to the caller and hard to test individual steps.

A cleaner split: introduce a `Config` struct (path + parsed doc) instantiated separately,
with `App::new(config: Config)` focusing only on UI state initialisation.

---
