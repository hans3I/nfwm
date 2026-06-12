# nfwm

`nfwm` is a Rust-based dynamic tiling window manager for Windows 10/11.

Current direction:
- headless, config-first runtime
- no tray, settings UI, overlay, or toast dependency
- background process controlled by CLI and runtime files under `%AppData%\nfwm`

## Status

This repo is still under active development.

What exists today:
- tiling core, layout engine, placement engine
- virtual desktop and multi-monitor foundations
- config-first runtime with `config.jsonc`
- CLI control surface
- migration from legacy FancyWM settings when possible

What is still incomplete:
- hotkey wiring into the live runtime
- production-grade Win32 edge-case handling for every app
- polished install/update experience

## Getting Started

Requirements:
- Windows 10/11
- Rust toolchain
- PowerShell 7+

Build:

```powershell
cd rust
cargo build -p nfwm-app
```

Run the runtime launcher:

```powershell
cd rust
cargo run -p nfwm-app
```

Important:
- `nfwm` starts a background runtime and returns immediately.
- This is expected. It is not a foreground UI app.

Check status:

```powershell
cd rust
cargo run -p nfwm-app -- status
```

Run attached in the foreground for debugging:

```powershell
cd rust
cargo run -p nfwm-app -- run
```

## CLI Usage

From the `rust/` directory:

```powershell
cargo run -p nfwm-app -- --help
```

Current commands:
- `nfwm` or `cargo run -p nfwm-app`:
  start the runtime if missing, otherwise ping the existing runtime
- `nfwm run`:
  run the runtime in the foreground
- `nfwm status`:
  print runtime PID, config path, hotkey count, and last reload result
- `nfwm reload`:
  re-read `config.jsonc`
- `nfwm stop`:
  stop the running runtime
- `nfwm action <name>`:
  queue one action for the runtime
- `nfwm diagnose`:
  enumerate windows/displays and log classification info
- `nfwm shadow`:
  compute layout and placements without moving windows

Examples:

```powershell
cargo run -p nfwm-app -- status
cargo run -p nfwm-app -- reload
cargo run -p nfwm-app -- action refresh
cargo run -p nfwm-app -- action split-horizontal
cargo run -p nfwm-app -- stop
```

## Config File

Location:

```text
%AppData%\nfwm\config.jsonc
```

The file is created automatically on first start.

The runtime accepts JSONC, so comments are allowed.

Top-level sections:
- `version`
- `general`
- `hotkeys`
- `ignore`
- `display`
- `behavior`
- `theme`

Example:

```jsonc
{
  "version": 1,
  "general": {
    "log_level": "info"
  },
  "hotkeys": {
    "activation_key": "Shift+Win",
    "activation_mode": "hold",
    "bindings": {
      "split-horizontal": "Shift+H",
      "split-vertical": "Shift+V",
      "stack": "Shift+S",
      "float": "Shift+F",
      "move-focus-left": "Shift+Left",
      "move-focus-right": "Shift+Right"
    }
  },
  "ignore": {
    "process_names": ["Taskmgr"],
    "class_names": ["RAIL_WINDOW"]
  },
  "display": {
    "multi_monitor": true
  },
  "behavior": {
    "poll_interval_ms": 750,
    "window_padding": 4,
    "panel_height": 18,
    "show_focus": false
  },
  "theme": {
    "mode": "none",
    "override_accent_color": false,
    "custom_accent_color": "#0064FFFF"
  }
}
```

Apply config changes:

```powershell
cargo run -p nfwm-app -- reload
```

Reload behavior:
- valid config: runtime applies supported live changes
- invalid config: runtime keeps the last known-good config and reports failure

## Runtime Directory

All runtime files live under:

```text
%AppData%\nfwm
```

Important files:
- `config.jsonc`: user configuration
- `runtime-status.json`: runtime status snapshot
- `runtime.lock`: single-instance guard
- `migration-report.txt`: legacy migration outcome
- `nfwm.log`: runtime log file
- `nfwm-crash-*.log`: panic/crash logs

## IPC / Control Plane

Current runtime control is file-based.

Location:

```text
%AppData%\nfwm
```

Current control files:
- `reload.signal`:
  created by `nfwm reload`
- `stop.signal`:
  created by `nfwm stop`
- `actions\*.cmd`:
  created by `nfwm action <name>`
- `runtime-status.json`:
  read by `nfwm status`

Current action names:
- `split-horizontal`
- `split-vertical`
- `stack`
- `float`
- `move-focus-left`
- `move-focus-right`
- `move-focus-up`
- `move-focus-down`
- `swap-left`
- `swap-right`
- `swap-up`
- `swap-down`
- `move-window-left`
- `move-window-right`
- `move-window-up`
- `move-window-down`
- `pull-up`
- `resize-left`
- `resize-right`
- `resize-up`
- `resize-down`
- `start`
- `stop`
- `discover`
- `refresh`
- `toggle`

Note:
- `nfwm-win32` also contains `WM_COPYDATA` helpers, but the runtime currently uses the file-based control path above.

## Legacy Migration

On first start, if `config.jsonc` does not exist, `nfwm` tries to import legacy settings from:

```text
%AppData%\FancyWM\settings.json
```

Behavior:
- successful import: writes a migrated `config.jsonc`
- failed import: writes a fresh default `config.jsonc`
- legacy file is never modified

This keeps rollback to the legacy app safe.

## Install

User-local install:

```powershell
scripts/install-user.ps1
```

Remove user-local install:

```powershell
scripts/uninstall-user.ps1
```

Install startup shortcut:

```powershell
scripts/install-startup-task.ps1
```

Remove startup shortcut:

```powershell
scripts/uninstall-startup-task.ps1
```

Build a release ZIP:

```powershell
scripts/package-release.ps1
```

## Diagnostics

Useful commands:

```powershell
cargo run -p nfwm-app -- diagnose
cargo run -p nfwm-app -- shadow
cargo run -p nfwm-app -- status
```

Tail the runtime log:

```powershell
Get-Content "$env:APPDATA\nfwm\nfwm.log" -Wait
```

## Contributing

Start with the docs in this repo:
- `docs/architecture.md`
- `docs/feature-parity-checklist.md`
- `docs/risk-register.md`
- `phase-10.md`, `phase-11.md`, `phase-12.md`

Project structure:
- `rust/nfwm-core`: pure tiling logic and config model
- `rust/nfwm-win32`: Win32 integration and unsafe boundary
- `rust/nfwm-app`: runtime, CLI, bootstrapping, control flow
- `rust/nfwm-ui`: deferred UI crate, currently not part of the runtime path

Before sending changes, run:

```powershell
cd rust
cargo fmt
cargo test --all
cargo clippy --all-targets
```

Useful repo scripts:
- `scripts/smoke-test.ps1`
- `scripts/package-release.ps1`

Manual QA docs:
- `docs/manual-qa-checklist.md`
- `docs/beta-release-plan.md`
- `docs/cutover-checklist.md`

Contribution expectations:
- keep `unsafe` isolated to `nfwm-win32`
- prefer small, testable changes
- preserve the config-first runtime direction unless explicitly changing plan/docs
- update docs when behavior or workflows change
