# nfwm Rewrite Charter

## Purpose

This charter authorizes the incremental rewrite of the legacy window manager into a Rust-native Windows application. The project name is **nfwm** (not FancyWM).

## Scope

- **In scope**: Windows 10/11 dynamic tiling window manager with feature parity to the legacy app.
- **Out of scope for v1**: Linux/macOS support, CSS theme engine, animation.
- **Deferred**: CSS theme engine full parity (decision in Phase 11).

## Strategy: Incremental Replacement

1. Keep the legacy app buildable and runnable as the reference.
2. Build Rust in **shadow mode** (read-only window discovery) until layout parity is proven.
3. Only allow Rust to move windows after shadow mode passes manual validation.
4. Retire the legacy app only after feature parity and real-desktop stability are proven.

## Language

- **Target**: Rust (Edition 2021)
- **MSRV**: TBD (likely latest stable)
- **Win32 bindings**: `windows` crate preferred, `windows-sys` as fallback
- **Architecture**: Cargo workspace with `nfwm-core`, `nfwm-win32`, `nfwm-ui`, `nfwm-app`

## Data Preservation

- `%AppData%\FancyWM` is the current legacy runtime data directory.
- Migration to `%AppData%\nfwm` will be designed in Phase 11.
- No existing legacy settings files will be modified during development.

## Success Criteria

- The Rust app can discover, classify, and tile windows without user-visible regressions.
- Existing keybindings and workflows are preserved or explicitly documented as changed.
- The app can be packaged as a standalone ZIP and optionally MSIX.
- CI runs `cargo fmt`, `cargo clippy`, and `cargo test`.

## Governance

- This charter is reviewed at the end of each phase.
- Major scope changes require a new decision record.
- The legacy app remains the behavior reference until the Rust app is stable.
