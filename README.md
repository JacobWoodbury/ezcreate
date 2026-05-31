# ezcreate (Bevy)

Townscaper-style modular building sandbox — Bevy port of the Godot **builder** project.

## New here?

**[docs/GETTING_STARTED.md](docs/GETTING_STARTED.md)** — full first-time setup, troubleshooting, and an in-app walkthrough.

### Quick setup

| Platform | Command |
|----------|---------|
| **Windows** (PowerShell) | `.\scripts\setup.ps1` |
| **macOS** (Terminal) | `chmod +x scripts/*.sh && ./scripts/setup-mac.sh` |
| **Linux** | `chmod +x scripts/*.sh && ./scripts/setup.sh` |

Then run:

```sh
cargo run
```

First compile can take **10–30+ minutes**; later runs are much faster.

### Prerequisites (summary)

- [Rust](https://rustup.rs) (stable)
- **Windows:** [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++** (provides `link.exe`)
- **macOS:** Xcode Command Line Tools (`xcode-select --install`) — see [Getting started → macOS](docs/GETTING_STARTED.md#macos)
- **Linux:** `build-essential` or equivalent

## Controls (short)

| Input | Action |
|-------|--------|
| `1` / `2` / `3` | Place / Select / Paint |
| Tab | Toggle Place ↔ Select |
| LMB | Place / select / paint face |
| Q / E | Rotate |
| Del | Delete selection |
| Alt + RMB | Delete block under cursor |
| Ctrl+Z / Ctrl+Y | Undo / redo (macOS: **Control**, not Command) |
| WASD + RMB drag + scroll | Camera |

Rebind keys in **Settings → Keybindings**. Full list: [PORT.md](PORT.md).

## Project layout

| Path | Description |
|------|-------------|
| `src/` | Game code (Bevy plugins, systems, UI) |
| `assets/mods/` | Built-in mod packs |
| `docs/` | Formats, porting notes, **getting started** |
| `scripts/` | `setup.ps1`, `setup-mac.sh`, `setup.sh`, `run.ps1`, `run.sh` |

## Mods

- **Built-in:** `assets/mods/sample_pack/mod.json`
- **User mods:** OS app data folder — see [GETTING_STARTED.md](docs/GETTING_STARTED.md#mods-and-data-folders)

## Docs

- [Getting started](docs/GETTING_STARTED.md) — setup scripts + walkthrough
- [PORT.md](PORT.md) — port status and controls
- [docs/data_formats.md](docs/data_formats.md) — `mod.json`, sections
- [docs/godot_gotchas.md](docs/godot_gotchas.md) — Godot → Bevy notes

## Source

- **Godot reference:** `~/Documents/builder` (original project)
- **Port spec:** `PORT.md` and `docs/`
