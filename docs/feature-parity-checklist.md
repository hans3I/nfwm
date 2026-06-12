# Feature Parity Checklist

Based on the legacy app. Items are checked off when the Rust version matches or exceeds the legacy behavior.

## Core Tiling

- [ ] Dynamic tiling: windows are automatically arranged
- [ ] Split panels: horizontal and vertical splits
- [ ] Stack panels: tabbed/stacked windows
- [ ] Window nodes: wrapping native windows
- [ ] Placeholder nodes: temporary pending locations
- [ ] Measure/arrange layout pass
- [ ] Min/max size constraint handling
- [ ] Flex sizing for panels
- [ ] Unsatisfiable-constraint handling (float/reject instead of overlap)
- [ ] DesktopTree per virtual desktop
- [ ] Window discovery and registration
- [ ] Original position restoration
- [ ] Collapse behavior when windows close

## Commands / Actions

- [ ] Split horizontally
- [ ] Split vertically
- [ ] Stack focused windows
- [ ] Float focused window
- [ ] Toggle tiling for a desktop
- [ ] Move focus by direction (left/right/up/down)
- [ ] Swap focus by direction
- [ ] Move window by direction
- [ ] Pull up focused window
- [ ] Resize panels (increase/decrease)
- [ ] Start tiling
- [ ] Stop tiling
- [ ] Discover windows
- [ ] Refresh layout
- [ ] Resolve focused window
- [ ] Resolve closest window
- [ ] Resolve managed bounds
- [ ] Pending intents (split/stack/group operations)

## Window Management

- [ ] Window enumeration (top-level windows)
- [ ] Window classification (title, class, process, styles)
- [ ] Exclusion matcher (process/class ignore lists)
- [ ] Floating decisions: dialogs, minimized, maximized, topmost, pinned, unresizable, too small
- [ ] Window registry updates on appearance/disappearance/move/minimize/maximize/focus change
- [ ] Original position tracking
- [ ] Placement failure handling
- [ ] Multi-monitor support (single-display and multi-display modes)
- [ ] Display connect/disconnect handling
- [ ] DPI awareness
- [ ] Work area respect

## Virtual Desktop

- [ ] Track current virtual desktop
- [ ] Track windows by virtual desktop
- [ ] Register/unregister desktop state
- [ ] Preserve separate tiling state per desktop
- [ ] Windows 10/11 compatibility

## Input / Hotkeys

- [ ] Global low-level keyboard hooks
- [ ] Direct hotkeys (activation key + action key)
- [ ] Command sequence mode
- [ ] Caps Lock activation (optional)
- [ ] Modifier-assisted window moving
- [ ] Low-level mouse handling
- [ ] Keybinding parser and serializer
- [ ] Default keybindings preserved

## IPC / CLI

- [ ] `--help` and `--version` CLI output
- [ ] `--action NAME` dispatch to running instance
- [ ] Local IPC mechanism (WM_COPYDATA equivalent or better)
- [ ] Single-instance enforcement

## UI

- [ ] Tray icon and menu
- [ ] Settings UI
- [ ] Keybinding editor
- [ ] Overlay windows (tiling previews, focus indication)
- [ ] Toast notifications
- [ ] Start/stop controls from tray
- [ ] About/help dialogs
- [ ] Update check

## Settings / State

- [ ] Settings JSON persistence
- [ ] Keybindings storage
- [ ] Ignore rules (process/class)
- [ ] Multi-monitor mode toggle
- [ ] Animation toggle
- [ ] Focus behavior settings
- [ ] Color/accent settings
- [ ] Update check settings
- [ ] Interaction behavior settings
- [ ] Theme selection (basic, even if CSS engine is deferred)

## Logging / Diagnostics

- [ ] Structured logging (Serilog equivalent)
- [ ] Log file rotation
- [ ] Crash dump/diagnostic logging
- [ ] Shadow mode diagnostics

## Packaging

- [ ] Loose ZIP release
- [ ] MSIX/Desktop Bridge package (decision in Phase 12)
- [ ] Startup task
- [ ] App execution alias
- [ ] Versioning
- [ ] Release notes
- [ ] winget distribution (decision in Phase 12)

## Tests

- [ ] Layout engine unit tests
- [ ] Flex constraint tests
- [ ] Command tests with fake windows
- [ ] Settings migration tests
- [ ] CI workflow for Rust

## Deferred / Optional

- [ ] CSS theme engine (decision in Phase 11)
- [ ] Animation engine (decision in Phase 07)
- [ ] Microsoft Store distribution (decision in Phase 12)
