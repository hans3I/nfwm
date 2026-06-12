# Risk Register

## R1: WPF UI Replacement Cost

- **Description**: Replacing legacy UI settings, overlay, and tray UI may be the largest single part of the rewrite.
- **Impact**: High
- **Likelihood**: High
- **Mitigation**: Choose a minimal UI framework first (Phase 10). Defer settings UI to a config file or simple dialog. Build tray-only MVP.
- **Owner**: Phase 10 lead

## R2: Win32 Edge Cases

- **Description**: Treating Win32 edge cases as library problems instead of product behavior. Windows lie about sizes, styles, and constraints.
- **Impact**: High
- **Likelihood**: High
- **Mitigation**: Build a diagnostic tool (Phase 03) that records real window behavior. Test against common app types: terminal, browser, IDE, UWP, elevated.
- **Owner**: Phase 03/07 lead

## R3: Virtual Desktop API Instability

- **Description**: Virtual desktop APIs differ across Windows 10/11 builds and may require COM or undocumented interfaces.
- **Impact**: High
- **Likelihood**: Medium
- **Mitigation**: Validate early in Phase 03. Build fallback behavior: if virtual desktop APIs fail, operate on a single-desktop basis. Document compatibility matrix.
- **Owner**: Phase 08 lead

## R4: Hook / Thread Safety

- **Description**: Global hooks can destabilize input or leave hooks registered if threads crash.
- **Impact**: High
- **Likelihood**: Medium
- **Mitigation**: Isolate hooks in a dedicated thread with a watchdog. Use `Drop` to guarantee cleanup. Test hook install/uninstall cycles aggressively.
- **Owner**: Phase 09 lead

## R5: Layout Behavior Loss

- **Description**: Subtle layout behavior encoded in legacy tiling service types may be lost during translation.
- **Impact**: Medium
- **Likelihood**: High
- **Mitigation**: Use legacy app as reference. Build shadow mode first. Compare Rust layout decisions against legacy decisions on the same window set.
- **Owner**: Phase 04/06 lead

## R6: Multi-Monitor Complexity

- **Description**: Multi-monitor behavior multiplies placement and focus edge cases.
- **Impact**: Medium
- **Likelihood**: High
- **Mitigation**: Start with single-monitor mode. Build multi-display tiling facade only after single-monitor is stable.
- **Owner**: Phase 08 lead

## R7: DPI / Scaling Bugs

- **Description**: DPI conversions can produce off-by-one or wrong-monitor placement bugs.
- **Impact**: Medium
- **Likelihood**: Medium
- **Mitigation**: Test on mixed-DPI setups. Log all coordinate conversions. Use physical pixels internally where possible.
- **Owner**: Phase 07 lead

## R8: Settings Migration

- **Description**: Reusing the exact settings file too early can corrupt user data.
- **Impact**: Medium
- **Likelihood**: Medium
- **Mitigation**: Use a separate test settings path during development. Implement versioned migration in Phase 11. Never write to the legacy app settings path.
- **Owner**: Phase 11 lead

## R9: Elevated Window Handling

- **Description**: Elevated windows may not be controllable unless the Rust app is elevated.
- **Impact**: Medium
- **Likelihood**: Medium
- **Mitigation**: Support elevation marker file (like legacy `administrator-mode`). Document when elevation is needed. Do not auto-elevate without user consent.
- **Owner**: Phase 03/05 lead

## R10: Monolith Tendency

- **Description**: The first executable may become a monolith like the legacy coordinator.
- **Impact**: Medium
- **Likelihood**: Medium
- **Mitigation**: Enforce crate boundaries. Core crates must not depend on UI or Win32. Use traits for OS integration. Code review for dependency violations.
- **Owner**: Phase 02 lead

## R11: Packaging Late Failure

- **Description**: MSIX/full-trust packaging may fail late if not tested early.
- **Impact**: Low
- **Likelihood**: Medium
- **Mitigation**: Build packaging scripts in Phase 12. Test MSIX packaging early. Have a fallback to loose ZIP.
- **Owner**: Phase 12 lead

## R12: Animation Instability

- **Description**: Animation can make native event storms worse if implemented too early.
- **Impact**: Low
- **Likelihood**: Medium
- **Mitigation**: Ship v1 without animation (Phase 07). Add animation only after core stability is proven.
- **Owner**: Phase 07 lead

## R13: Theme Engine Cost

- **Description**: CSS theme engine parity may be expensive and lower priority than tiling parity.
- **Impact**: Low
- **Likelihood**: Medium
- **Mitigation**: Defer CSS theme engine to Phase 11. Support basic color/accent settings in v1.
- **Owner**: Phase 11 lead

## R14: UWP / Modern App Handling

- **Description**: UWP and modern apps have different windowing models that may resist tiling.
- **Impact**: Medium
- **Likelihood**: High
- **Mitigation**: Classify UWP apps correctly during discovery. Allow them to float if they cannot be resized.
- **Owner**: Phase 05 lead

## R15: Keyboard Layout Differences

- **Description**: Keyboard layout differences may affect key descriptions and bindings.
- **Impact**: Low
- **Likelihood**: Medium
- **Mitigation**: Use virtual key codes, not characters. Test on multiple keyboard layouts.
- **Owner**: Phase 09 lead
