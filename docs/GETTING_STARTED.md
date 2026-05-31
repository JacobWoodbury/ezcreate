# Getting started with ezcreate

Townscaper-style modular building sandbox (Rust + [Bevy](https://bevyengine.org)). This guide covers **first-time setup**, **running the game**, and a short **in-app walkthrough**.

---

## What you need

| Requirement | Windows | macOS / Linux |
|-------------|---------|----------------|
| Rust (stable) | [rustup.rs](https://rustup.rs) — use **MSVC** host (`x86_64-pc-windows-msvc`) | [rustup.rs](https://rustup.rs) |
| C++ linker | **Visual Studio Build Tools 2022** with **Desktop development with C++** | Xcode CLI tools (macOS) or `build-essential` (Linux) |
| GPU | Vulkan-capable GPU (NVIDIA/AMD/Intel); see [troubleshooting](#troubleshooting) | Same |

---

## First-time setup (automated)

Clone the repo, then run the setup script for your OS.

### Windows (recommended: PowerShell)

```powershell
cd path\to\Ezcreate
.\scripts\setup.ps1
```

If scripts are blocked:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup.ps1
```

The script checks Rust, MSVC `link.exe`, and runs `cargo check`.

### macOS (Terminal)

Open **Terminal** (or iTerm) from the repo folder:

```bash
cd path/to/Ezcreate
chmod +x scripts/*.sh
./scripts/setup-mac.sh
```

`./scripts/setup.sh` also works on macOS — it forwards to `setup-mac.sh`.

The macOS script checks **Xcode Command Line Tools** (`clang` / linker), **Rust**, and runs `cargo check`.

**Apple Silicon (M1/M2/M3/M4):** use the default rustup host (`aarch64-apple-darwin`). No extra target is needed for a normal build.

**Intel Mac:** host is `x86_64-apple-darwin`.

If Command Line Tools are missing, install them:

```bash
xcode-select --install
```

When the installer finishes, run `./scripts/setup-mac.sh` again.

### Linux

```bash
cd path/to/Ezcreate
chmod +x scripts/*.sh
./scripts/setup.sh
```

On Linux the script checks Rust, a C compiler (`cc`), and runs `cargo check`.

> **Note:** On Windows, use PowerShell and `setup.ps1`, not Git Bash, for `cargo build` / `cargo run`.

---

## macOS

This section is the mac-specific companion to the rest of the guide (setup paths, keys, and data folders).

### Prerequisites

| Item | How to get it |
|------|----------------|
| **Xcode Command Line Tools** | `xcode-select --install` (or full Xcode from the App Store) |
| **Rust** | [rustup.rs](https://rustup.rs) (install via the curl script on that page), then open a **new** Terminal window |
| **GPU** | Built-in or discrete GPU with Metal (Bevy uses wgpu; macOS uses Metal) |

### One-command setup

```bash
chmod +x scripts/setup-mac.sh scripts/run.sh
./scripts/setup-mac.sh
./scripts/run.sh
```

### Manual setup (macOS)

1. **Command Line Tools** — run `xcode-select --install` and complete the dialog. Verify:

   ```bash
   xcode-select -p
   clang --version
   ```

2. **Rust** — install via rustup, then verify:

   ```bash
   rustc --version
   cargo --version
   rustc -vV | grep host
   ```

3. **Build and run** from the repo root:

   ```bash
   cargo check
   cargo run
   ```

First full compile often takes **10-30+ minutes**; later runs are much faster.

### macOS controls and walkthrough notes

The in-app walkthrough below applies on macOS too. A few differences:

| Topic | macOS |
|-------|--------|
| **Undo / redo** | Hold **Control** (not Command) and press **Z** / **Y** — same as the Windows build today |
| **Delete selection** | **Delete** or **Fn+Delete** (forward delete on some keyboards) |
| **Alt + right-click** delete under cursor | Use **Option** + right-click (trackpad: enable secondary click in System Settings, or hold Control while clicking) |
| **Camera orbit** | Right-click or **Control**-click and drag |
| **Scroll zoom** | Two-finger scroll on trackpad, or mouse wheel |
| **Saved data** | `~/Library/Application Support/ezcreate/` (mods, stamps) |

Rebind keys in **Settings → Keybindings** if Control+Z feels awkward; you can assign other keys there.

---

## First-time setup (manual)

### 1. Install Rust

```bash
# Visit https://rustup.rs or:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:** after install, prefer the MSVC toolchain:

```powershell
rustup default stable-x86_64-pc-windows-msvc
```

Verify:

```bash
rustc --version
cargo --version
```

### 2. Windows only — Visual Studio Build Tools

Rust on Windows needs Microsoft's C++ linker (`link.exe`).

1. Download [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
2. Run the installer and enable **Desktop development with C++**.
3. Restart the terminal.

Optional silent install (Admin PowerShell, from a folder containing `vs_buildtools.exe`):

```powershell
.\vs_buildtools.exe --passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
```

Verify (PowerShell):

```powershell
where.exe link
# Should show a path under Microsoft Visual Studio\...\bin\Hostx64\x64\link.exe
```

### 3. Build the project

From the repo root:

```bash
cargo check   # faster sanity check
cargo run     # build and launch
```

**First full build** often takes **10–30+ minutes** (Bevy + dependencies). Later builds are much faster.

### 4. Run

```bash
cargo run
```

Or use helper scripts:

```powershell
.\scripts\run.ps1    # Windows
```

```bash
./scripts/run.sh     # macOS / Linux
```

---

## In-app walkthrough

When the window opens you should see a **top bar** (modes), a **left library**, and a **3D view** with a green ground plane.

### 1. Place your first block

1. Press **`1`** or click **Place** (top bar).
2. In the **Library** sidebar, click a block (e.g. from the sample pack).
3. Move the mouse over the ground — a **green ghost** shows where the block will go.
4. **Left-click** to place.
5. Press **`Q`** / **`E`** (or sidebar buttons) to rotate before placing.

### 2. Select and delete

1. Press **`2`** or click **Select**.
2. **Click** a block to select it, or **drag** a rectangle to marquee-select several.
3. Press **Delete** to remove the selection.
4. **Q** / **E** rotate the whole selection.

### 3. Paint a block face

1. Press **`3`** or click **Paint**.
2. In the sidebar **Stamp editor**:
   - Pick a **brush color**.
   - Click cells in the **pixel grid** to design your stamp.
   - Switch to **Apply** (not Edit).
3. Hover a block face — you should see a **preview** of the stamp on that face.
4. **Left-click** to apply.
5. **Clear** (in the stamp editor) makes the grid empty; then painting uses a **solid brush color** instead of the grid.

**Saved stamps:** name your stamp and click **Save**, or use **Reload saved** and pick from the list below.

### 4. Undo and settings

- **Ctrl+Z** / **Ctrl+Y** — undo / redo placement and paint (on **macOS**, use the **Control** key, not Command).
- **Settings** (gear, top-right) — **General** (grid, overlap, shift-select) and **Keybindings** (rebind keys).

### 5. Camera

| Input | Action |
|-------|--------|
| **W A S D** | Pan |
| **Right-drag** | Orbit |
| **Scroll** | Zoom |

Full control list: [PORT.md](../PORT.md).

---

## Mods and data folders

| Location | Purpose |
|----------|---------|
| `assets/mods/` | Built-in mods shipped with the repo (`sample_pack`) |
| OS data dir | User mods, saved stamps, blueprints |

User data paths (via `dirs` crate):

| OS | Example path |
|----|----------------|
| Windows | `%APPDATA%\ezcreate\` |
| macOS | `~/Library/Application Support/ezcreate/` |
| Linux | `~/.local/share/ezcreate/` |

Under that folder:

- `mods/` — drop-in mod folders with `mod.json`
- `stamps/` — saved paint stamps (JSON)

Mod format: [data_formats.md](data_formats.md).

---

## Troubleshooting

### `link.exe` not found (Windows)

Install **Build Tools for C++** (see above). Run `cargo` from **PowerShell** or **x64 Native Tools Command Prompt**, not plain Git Bash (Git Bash can pick the wrong `link.exe`).

### `link: extra operand` in Git Bash

Same as above — use PowerShell for `cargo build` / `cargo run`.

### Vulkan / Epic overlay errors in the log

Harmless warnings about missing Epic overlay JSON files. If the game runs and shows your GPU in the log, you can ignore them. To reduce noise, remove stale Vulkan layer registry entries or use a different graphics backend (future option).

### Paint appears on the wrong place

Ensure you are in **Apply** mode in the Paint sidebar and that the stamp grid has colored pixels (or use **Clear** for solid brush color).

### Build is very slow

Normal for the first compile. Dev profile optimizes dependencies; wait once, then use `cargo run` for day-to-day work.

### macOS: `xcode-select` errors or missing `clang`

Install or reset Command Line Tools:

```bash
xcode-select --install
# If you use full Xcode:
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
```

Then run `./scripts/setup-mac.sh` again.

### macOS: Rust installed but `cargo` not found

Open a **new** Terminal window after rustup install, or run:

```bash
source "$HOME/.cargo/env"
```

### macOS: window does not open / Metal errors

Update macOS and Xcode Command Line Tools. Run from Terminal (not only from an IDE) to see full logs: `RUST_LOG=info cargo run`.

### Re-run setup check

```powershell
.\scripts\setup.ps1
```

```bash
./scripts/setup-mac.sh    # macOS
./scripts/setup.sh        # Linux (macOS forwards to setup-mac.sh)
```

---

## Next steps

- Port status and roadmap: [PORT.md](../PORT.md)
- Godot parity notes: [godot_gotchas.md](godot_gotchas.md)
- Rotation rules for sections: [rotation_invariants.md](rotation_invariants.md)
