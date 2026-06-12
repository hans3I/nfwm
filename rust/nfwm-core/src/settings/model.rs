//! Settings model: user-facing configuration.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const CURRENT_SETTINGS_VERSION: u32 = 1;

/// User settings for nfwm loaded from `%AppData%\nfwm\config.jsonc`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "current_settings_version")]
    pub version: u32,
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub hotkeys: HotkeySettings,
    #[serde(default)]
    pub ignore: IgnoreSettings,
    #[serde(default)]
    pub display: DisplaySettings,
    #[serde(default)]
    pub behavior: BehaviorSettings,
    #[serde(default)]
    pub theme: ThemeSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: CURRENT_SETTINGS_VERSION,
            general: GeneralSettings::default(),
            hotkeys: HotkeySettings::default(),
            ignore: IgnoreSettings::default(),
            display: DisplaySettings::default(),
            behavior: BehaviorSettings::default(),
            theme: ThemeSettings::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default = "default_log_level", alias = "logLevel")]
    pub log_level: String,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeySettings {
    #[serde(default, alias = "activationKey")]
    pub activation_key: Option<String>,
    #[serde(default = "default_activation_mode", alias = "activationMode")]
    pub activation_mode: String,
    #[serde(default = "default_hotkey_bindings")]
    pub bindings: BTreeMap<String, String>,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            activation_key: None,
            activation_mode: default_activation_mode(),
            bindings: default_hotkey_bindings(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IgnoreSettings {
    #[serde(default, alias = "processIgnoreList")]
    pub process_names: Vec<String>,
    #[serde(default, alias = "classIgnoreList")]
    pub class_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplaySettings {
    #[serde(default = "default_true", alias = "multiMonitorSupport")]
    pub multi_monitor: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            multi_monitor: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviorSettings {
    #[serde(default = "default_true", alias = "autoReloadOnCommand")]
    pub auto_reload_on_command: bool,
    #[serde(default = "default_poll_interval_ms", alias = "pollIntervalMs")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_true", alias = "allocateNewPanelSpace")]
    pub allocate_new_panel_space: bool,
    #[serde(default = "default_false", alias = "animateWindowMovement")]
    pub animate_window_movement: bool,
    #[serde(default = "default_false", alias = "autoCollapsePanels")]
    pub auto_collapse_panels: bool,
    #[serde(default = "default_auto_split_count", alias = "autoSplitCount")]
    pub auto_split_count: u32,
    #[serde(default = "default_true", alias = "delayReposition")]
    pub delay_reposition: bool,
    #[serde(default = "default_false", alias = "autoFloatNewWindows")]
    pub auto_float_new_windows: bool,
    #[serde(default = "default_false", alias = "modifierMoveWindow")]
    pub modifier_move_window: bool,
    #[serde(default = "default_false", alias = "modifierMoveWindowAutoFocus")]
    pub modifier_move_window_auto_focus: bool,
    #[serde(default = "default_window_padding", alias = "windowPadding")]
    pub window_padding: i32,
    #[serde(default = "default_panel_height", alias = "panelHeight")]
    pub panel_height: i32,
    #[serde(default = "default_panel_font_size", alias = "panelFontSize")]
    pub panel_font_size: i32,
    #[serde(default = "default_false", alias = "showFocus")]
    pub show_focus: bool,
    #[serde(default = "default_true", alias = "showFocusDuringAction")]
    pub show_focus_during_action: bool,
    #[serde(default = "default_true", alias = "showContextHints")]
    pub show_context_hints: bool,
    #[serde(default = "default_true", alias = "soundOnFailure")]
    pub sound_on_failure: bool,
    #[serde(default = "default_true", alias = "checkForUpdates")]
    pub check_for_updates: bool,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            auto_reload_on_command: true,
            poll_interval_ms: default_poll_interval_ms(),
            allocate_new_panel_space: true,
            animate_window_movement: false,
            auto_collapse_panels: false,
            auto_split_count: default_auto_split_count(),
            delay_reposition: true,
            auto_float_new_windows: false,
            modifier_move_window: false,
            modifier_move_window_auto_focus: false,
            window_padding: default_window_padding(),
            panel_height: default_panel_height(),
            panel_font_size: default_panel_font_size(),
            show_focus: false,
            show_focus_during_action: true,
            show_context_hints: true,
            sound_on_failure: true,
            check_for_updates: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeSettings {
    #[serde(default = "default_theme_mode")]
    pub mode: String,
    #[serde(default = "default_false", alias = "overrideAccentColor")]
    pub override_accent_color: bool,
    #[serde(default = "default_accent_color", alias = "customAccentColor")]
    pub custom_accent_color: String,
    #[serde(default, alias = "legacyCustomCssPath")]
    pub legacy_custom_css_path: Option<String>,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            mode: default_theme_mode(),
            override_accent_color: false,
            custom_accent_color: default_accent_color(),
            legacy_custom_css_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsMigration {
    pub settings: Settings,
    pub notes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsParseError {
    #[error("invalid configuration: {0}")]
    Invalid(#[from] serde_json::Error),
}

impl Settings {
    pub fn parse_jsonc(input: &str) -> Result<Self, SettingsParseError> {
        Ok(Self::parse_jsonc_with_notes(input)?.settings)
    }

    pub fn parse_jsonc_with_notes(input: &str) -> Result<SettingsMigration, SettingsParseError> {
        let stripped = strip_jsonc_comments(input);
        let raw: RawSettings = serde_json::from_str(&stripped)?;
        Ok(migrate_raw_settings(raw))
    }

    pub fn from_legacy_json(input: &str) -> Result<SettingsMigration, SettingsParseError> {
        let raw: LegacySettings = serde_json::from_str(input)?;
        Ok(migrate_legacy_settings(raw))
    }

    pub fn effective_hotkey_count(&self) -> usize {
        self.hotkeys.bindings.len()
    }
}

pub fn default_settings_jsonc() -> String {
    render_settings_jsonc(
        &Settings::default(),
        &[
            "nfwm configuration".to_string(),
            "Edit this file and run `nfwm reload` to apply supported changes.".to_string(),
        ],
    )
}

pub fn render_settings_jsonc(settings: &Settings, comments: &[String]) -> String {
    let pretty = serde_json::to_string_pretty(settings).expect("settings serialize");
    let mut lines = comments
        .iter()
        .map(|line| format!("// {line}"))
        .collect::<Vec<_>>();
    lines.push(pretty);
    lines.push(String::new());
    lines.join("\n")
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawSettings {
    #[serde(default)]
    version: Option<u32>,
    #[serde(default)]
    general: GeneralSettings,
    #[serde(default)]
    hotkeys: HotkeySettings,
    #[serde(default)]
    ignore: IgnoreSettings,
    #[serde(default)]
    display: DisplaySettings,
    #[serde(default)]
    behavior: BehaviorSettings,
    #[serde(default)]
    theme: ThemeSettings,
    #[serde(default, alias = "processIgnoreList")]
    legacy_process_ignore_list: Option<Vec<String>>,
    #[serde(default, alias = "classIgnoreList")]
    legacy_class_ignore_list: Option<Vec<String>>,
    #[serde(default, alias = "multiMonitorSupport")]
    legacy_multi_monitor_support: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct LegacySettings {
    #[serde(default)]
    activation_hotkey: Option<String>,
    #[serde(default)]
    activate_on_caps_lock: bool,
    #[serde(default)]
    allocate_new_panel_space: Option<bool>,
    #[serde(default)]
    auto_collapse_panels: Option<bool>,
    #[serde(default)]
    auto_split_count: Option<u32>,
    #[serde(default)]
    delay_reposition: Option<bool>,
    #[serde(default)]
    auto_float_new_windows: Option<bool>,
    #[serde(default)]
    animate_window_movement: Option<bool>,
    #[serde(default)]
    modifier_move_window: Option<bool>,
    #[serde(default)]
    modifier_move_window_auto_focus: Option<bool>,
    #[serde(default)]
    window_padding: Option<i32>,
    #[serde(default)]
    panel_height: Option<i32>,
    #[serde(default)]
    panel_font_size: Option<i32>,
    #[serde(default)]
    show_focus: Option<bool>,
    #[serde(default)]
    show_focus_during_action: Option<bool>,
    #[serde(default)]
    override_accent_color: Option<bool>,
    #[serde(default)]
    custom_accent_color: Option<String>,
    #[serde(default)]
    keybindings: Option<Value>,
    #[serde(default)]
    process_ignore_list: Option<Vec<String>>,
    #[serde(default)]
    class_ignore_list: Option<Vec<String>>,
    #[serde(default)]
    show_context_hints: Option<bool>,
    #[serde(default)]
    multi_monitor_support: Option<bool>,
    #[serde(default)]
    sound_on_failure: Option<bool>,
    #[serde(default)]
    check_for_updates: Option<bool>,
}

fn migrate_raw_settings(raw: RawSettings) -> SettingsMigration {
    let input_version = raw.version.unwrap_or(0);
    let mut notes = Vec::new();

    if input_version == 0 {
        notes.push("Loaded unversioned config and normalized it to version 1.".to_string());
    } else if input_version > CURRENT_SETTINGS_VERSION {
        notes.push(format!(
            "Loaded future config version {input_version} leniently and normalized known fields to version {}.",
            CURRENT_SETTINGS_VERSION
        ));
    }

    let mut settings = Settings {
        version: CURRENT_SETTINGS_VERSION,
        general: raw.general,
        hotkeys: raw.hotkeys,
        ignore: raw.ignore,
        display: raw.display,
        behavior: raw.behavior,
        theme: raw.theme,
    };

    if let Some(process_names) = raw.legacy_process_ignore_list {
        settings.ignore.process_names = process_names;
        notes.push(
            "Normalized deprecated top-level processIgnoreList into ignore.process_names."
                .to_string(),
        );
    }
    if let Some(class_names) = raw.legacy_class_ignore_list {
        settings.ignore.class_names = class_names;
        notes.push(
            "Normalized deprecated top-level classIgnoreList into ignore.class_names.".to_string(),
        );
    }
    if let Some(multi_monitor) = raw.legacy_multi_monitor_support {
        settings.display.multi_monitor = multi_monitor;
        notes.push(
            "Normalized deprecated top-level multiMonitorSupport into display.multi_monitor."
                .to_string(),
        );
    }

    SettingsMigration { settings, notes }
}

fn migrate_legacy_settings(raw: LegacySettings) -> SettingsMigration {
    let mut settings = Settings::default();
    let mut notes = vec![
        "Imported legacy FancyWM settings into config.jsonc.".to_string(),
        "The original legacy settings file is left untouched for rollback safety.".to_string(),
    ];

    if raw.activate_on_caps_lock {
        settings.hotkeys.activation_key = Some("CapsLock".to_string());
        settings.hotkeys.activation_mode = "hold".to_string();
    } else if let Some(hotkey) = raw.activation_hotkey.as_deref() {
        if let Some(mapped) = map_legacy_activation_hotkey(hotkey) {
            settings.hotkeys.activation_key = Some(mapped);
            settings.hotkeys.activation_mode = "hold".to_string();
        }
    }

    settings.ignore.process_names = raw
        .process_ignore_list
        .unwrap_or(settings.ignore.process_names);
    settings.ignore.class_names = raw.class_ignore_list.unwrap_or(settings.ignore.class_names);
    settings.display.multi_monitor = raw
        .multi_monitor_support
        .unwrap_or(settings.display.multi_monitor);

    if let Some(value) = raw.allocate_new_panel_space {
        settings.behavior.allocate_new_panel_space = value;
    }
    if let Some(value) = raw.auto_collapse_panels {
        settings.behavior.auto_collapse_panels = value;
    }
    if let Some(value) = raw.auto_split_count {
        settings.behavior.auto_split_count = value;
    }
    if let Some(value) = raw.delay_reposition {
        settings.behavior.delay_reposition = value;
    }
    if let Some(value) = raw.auto_float_new_windows {
        settings.behavior.auto_float_new_windows = value;
    }
    if let Some(value) = raw.animate_window_movement {
        settings.behavior.animate_window_movement = value;
    }
    if let Some(value) = raw.modifier_move_window {
        settings.behavior.modifier_move_window = value;
    }
    if let Some(value) = raw.modifier_move_window_auto_focus {
        settings.behavior.modifier_move_window_auto_focus = value;
    }
    if let Some(value) = raw.window_padding {
        settings.behavior.window_padding = value;
    }
    if let Some(value) = raw.panel_height {
        settings.behavior.panel_height = value;
    }
    if let Some(value) = raw.panel_font_size {
        settings.behavior.panel_font_size = value;
    }
    if let Some(value) = raw.show_focus {
        settings.behavior.show_focus = value;
    }
    if let Some(value) = raw.show_focus_during_action {
        settings.behavior.show_focus_during_action = value;
    }
    if let Some(value) = raw.show_context_hints {
        settings.behavior.show_context_hints = value;
    }
    if let Some(value) = raw.sound_on_failure {
        settings.behavior.sound_on_failure = value;
    }
    if let Some(value) = raw.check_for_updates {
        settings.behavior.check_for_updates = value;
    }

    if let Some(value) = raw.override_accent_color {
        settings.theme.override_accent_color = value;
    }
    if let Some(value) = raw.custom_accent_color {
        settings.theme.custom_accent_color = value;
    }

    if let Some(keybindings) = raw.keybindings {
        let imported = import_legacy_keybindings(keybindings, &mut settings.hotkeys, &mut notes);
        if imported == 0 {
            notes.push("No compatible legacy keybindings were imported.".to_string());
        }
    }

    notes.push(
        "Theme engine parity is deferred; only basic accent/theme compatibility is preserved in the headless runtime schema."
            .to_string(),
    );

    SettingsMigration { settings, notes }
}

fn import_legacy_keybindings(
    keybindings: Value,
    hotkeys: &mut HotkeySettings,
    notes: &mut Vec<String>,
) -> usize {
    let mut imported = 0;
    let Some(map) = keybindings.as_object() else {
        notes.push(
            "Legacy keybindings were present but not in an importable object format.".to_string(),
        );
        return 0;
    };

    for (legacy_action, raw_binding) in map {
        let Some(action_name) = map_legacy_action(legacy_action) else {
            continue;
        };

        match parse_legacy_binding_value(raw_binding) {
            Some(Some(binding)) => {
                if let Some(converted) = convert_legacy_binding_string(&binding, hotkeys, notes) {
                    hotkeys.bindings.insert(action_name.to_string(), converted);
                    imported += 1;
                }
            }
            Some(None) => {
                hotkeys.bindings.remove(action_name);
                imported += 1;
            }
            None => notes.push(format!(
                "Skipped unsupported legacy keybinding format for action `{legacy_action}`."
            )),
        }
    }

    imported
}

fn parse_legacy_binding_value(value: &Value) -> Option<Option<String>> {
    if value.is_null() {
        return Some(None);
    }
    if let Some(text) = value.as_str() {
        return Some(Some(text.to_string()));
    }

    if let Some(object) = value.as_object() {
        let keys = object.get("Keys")?.as_array()?;
        let key_names = keys
            .iter()
            .filter_map(|entry| entry.as_str().map(ToString::to_string))
            .collect::<Vec<_>>();
        let is_direct = object
            .get("IsDirectMode")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let prefix = if is_direct { "" } else { "Activation " };
        return Some(Some(format!("{prefix}{}", key_names.join("+"))));
    }

    if let Some(array) = value.as_array() {
        let key_names = array
            .iter()
            .filter_map(|entry| entry.as_str().map(ToString::to_string))
            .collect::<Vec<_>>();
        return Some(Some(format!("Activation {}", key_names.join("+"))));
    }

    None
}

fn convert_legacy_binding_string(
    binding: &str,
    hotkeys: &mut HotkeySettings,
    notes: &mut Vec<String>,
) -> Option<String> {
    let mut value = binding.trim();
    let uses_activation = value.starts_with("Activation ");
    if uses_activation {
        value = value.trim_start_matches("Activation ").trim();
        if hotkeys.activation_key.is_none() {
            hotkeys.activation_key = Some("Shift+Win".to_string());
            hotkeys.activation_mode = "hold".to_string();
            notes.push(
                "Imported activation-based keybindings and defaulted the activation hotkey to Shift+Win because the legacy config did not specify one explicitly."
                    .to_string(),
            );
        }
    }

    let keys = value
        .split('+')
        .filter_map(|token| map_legacy_key_name(token.trim()))
        .collect::<Vec<_>>();

    if keys.is_empty() {
        return None;
    }

    Some(keys.join("+"))
}

fn map_legacy_activation_hotkey(value: &str) -> Option<String> {
    let normalized = value.replace('_', "+");
    if normalized.eq_ignore_ascii_case("None+None") || normalized.eq_ignore_ascii_case("Disabled") {
        return None;
    }
    let keys = normalized
        .split('+')
        .filter_map(|token| map_legacy_key_name(token.trim()))
        .collect::<Vec<_>>();
    if keys.is_empty() {
        None
    } else {
        Some(keys.join("+"))
    }
}

fn map_legacy_key_name(value: &str) -> Option<String> {
    let mapped = match value {
        "LeftShift" | "ShiftKey" | "Shift" => "Shift",
        "LeftCtrl" | "ControlKey" | "Ctrl" => "Ctrl",
        "LeftAlt" | "Menu" | "Alt" => "Alt",
        "LWin" | "Win" => "Win",
        "Return" | "Enter" => "Enter",
        "Capital" | "CapsLock" => "CapsLock",
        "OemCloseBrackets" => "CloseBracket",
        "OemOpenBrackets" => "OpenBracket",
        "OemSemicolon" => "Semicolon",
        "OemQuotes" => "Quote",
        "Escape" => "Escape",
        "Left" => "Left",
        "Right" => "Right",
        "Up" => "Up",
        "Down" => "Down",
        "F1" => "F1",
        "F2" => "F2",
        "F3" => "F3",
        "F4" => "F4",
        "F5" => "F5",
        "F6" => "F6",
        "F7" => "F7",
        "F8" => "F8",
        "F9" => "F9",
        "F10" => "F10",
        "F11" => "F11",
        "F12" => "F12",
        "D1" => "1",
        "D2" => "2",
        "D3" => "3",
        "D4" => "4",
        "D5" => "5",
        "D6" => "6",
        "D7" => "7",
        "D8" => "8",
        "D9" => "9",
        other if other.len() == 1 && other.chars().all(|ch| ch.is_ascii_alphanumeric()) => {
            return Some(other.to_ascii_uppercase())
        }
        _ => return None,
    };
    Some(mapped.to_string())
}

fn map_legacy_action(value: &str) -> Option<&'static str> {
    match value {
        "ToggleManager" => Some("toggle"),
        "RefreshWorkspace" => Some("refresh"),
        "MoveFocusLeft" => Some("move-focus-left"),
        "MoveFocusRight" => Some("move-focus-right"),
        "MoveFocusUp" => Some("move-focus-up"),
        "MoveFocusDown" => Some("move-focus-down"),
        "CreateHorizontalPanel" => Some("split-horizontal"),
        "CreateVerticalPanel" => Some("split-vertical"),
        "CreateStackPanel" => Some("stack"),
        "PullWindowUp" => Some("pull-up"),
        "ToggleFloatingMode" => Some("float"),
        "MoveLeft" => Some("move-window-left"),
        "MoveRight" => Some("move-window-right"),
        "MoveUp" => Some("move-window-up"),
        "MoveDown" => Some("move-window-down"),
        "SwapLeft" => Some("swap-left"),
        "SwapRight" => Some("swap-right"),
        "SwapUp" => Some("swap-up"),
        "SwapDown" => Some("swap-down"),
        "IncreaseWidth" | "IncreaseHeight" => Some("resize-right"),
        "DecreaseWidth" | "DecreaseHeight" => Some("resize-left"),
        _ => None,
    }
}

fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaping = false;

    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if escaping {
                escaping = false;
            } else if ch == '\\' {
                escaping = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            continue;
        }

        if ch == '/' {
            match chars.peek().copied() {
                Some('/') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if next == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut prev = '\0';
                    for next in chars.by_ref() {
                        if prev == '*' && next == '/' {
                            break;
                        }
                        prev = next;
                    }
                    continue;
                }
                _ => {}
            }
        }

        out.push(ch);
    }

    out
}

fn current_settings_version() -> u32 {
    CURRENT_SETTINGS_VERSION
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_activation_mode() -> String {
    "direct".to_string()
}

fn default_poll_interval_ms() -> u64 {
    750
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_auto_split_count() -> u32 {
    2
}

fn default_window_padding() -> i32 {
    4
}

fn default_panel_height() -> i32 {
    18
}

fn default_panel_font_size() -> i32 {
    12
}

fn default_theme_mode() -> String {
    "none".to_string()
}

fn default_accent_color() -> String {
    "#0064FFFF".to_string()
}

fn default_hotkey_bindings() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("float".to_string(), "Shift+F".to_string()),
        ("move-focus-down".to_string(), "Shift+Down".to_string()),
        ("move-focus-left".to_string(), "Shift+Left".to_string()),
        ("move-focus-right".to_string(), "Shift+Right".to_string()),
        ("move-focus-up".to_string(), "Shift+Up".to_string()),
        ("move-window-down".to_string(), "Shift+Alt+Down".to_string()),
        ("move-window-left".to_string(), "Shift+Alt+Left".to_string()),
        (
            "move-window-right".to_string(),
            "Shift+Alt+Right".to_string(),
        ),
        ("move-window-up".to_string(), "Shift+Alt+Up".to_string()),
        ("pull-up".to_string(), "Shift+U".to_string()),
        ("resize-left".to_string(), "Shift+Minus".to_string()),
        ("resize-right".to_string(), "Shift+Plus".to_string()),
        ("split-horizontal".to_string(), "Shift+H".to_string()),
        ("split-vertical".to_string(), "Shift+V".to_string()),
        ("stack".to_string(), "Shift+S".to_string()),
        ("swap-down".to_string(), "Shift+Ctrl+Down".to_string()),
        ("swap-left".to_string(), "Shift+Ctrl+Left".to_string()),
        ("swap-right".to_string(), "Shift+Ctrl+Right".to_string()),
        ("swap-up".to_string(), "Shift+Ctrl+Up".to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_jsonc_with_comments() {
        let input = r#"
        {
          // general settings
          "general": { "log_level": "debug" },
          "hotkeys": {
            "bindings": {
              "split-horizontal": "Shift+H"
            }
          }
        }
        "#;

        let settings = Settings::parse_jsonc(input).unwrap();
        assert_eq!(settings.general.log_level, "debug");
        assert_eq!(settings.version, CURRENT_SETTINGS_VERSION);
        assert_eq!(
            settings
                .hotkeys
                .bindings
                .get("split-horizontal")
                .map(String::as_str),
            Some("Shift+H")
        );
    }

    #[test]
    fn defaults_have_hotkeys() {
        let settings = Settings::default();
        assert!(settings.effective_hotkey_count() > 0);
    }

    #[test]
    fn generated_default_jsonc_mentions_reload() {
        let text = default_settings_jsonc();
        assert!(text.contains("nfwm reload"));
    }

    #[test]
    fn unknown_future_fields_are_ignored() {
        let input = r#"
        {
          "version": 99,
          "general": { "log_level": "trace", "future": true },
          "totallyFuture": { "x": 1 }
        }
        "#;
        let parsed = Settings::parse_jsonc_with_notes(input).unwrap();
        assert_eq!(parsed.settings.general.log_level, "trace");
        assert_eq!(parsed.settings.version, CURRENT_SETTINGS_VERSION);
        assert!(!parsed.notes.is_empty());
    }

    #[test]
    fn aliases_and_partial_config_work() {
        let input = r#"
        {
          "multiMonitorSupport": false,
          "processIgnoreList": ["Code"],
          "behavior": { "windowPadding": 12 }
        }
        "#;
        let settings = Settings::parse_jsonc(input).unwrap();
        assert!(!settings.display.multi_monitor);
        assert_eq!(settings.ignore.process_names, vec!["Code"]);
        assert_eq!(settings.behavior.window_padding, 12);
        assert_eq!(settings.behavior.panel_height, 18);
    }

    #[test]
    fn legacy_settings_migrate() {
        let input = r##"
        {
          "ActivationHotkey": "Shift_Win",
          "AnimateWindowMovement": true,
          "ModifierMoveWindow": true,
          "WindowPadding": 8,
          "ProcessIgnoreList": ["Taskmgr", "Code"],
          "ClassIgnoreList": ["RAIL_WINDOW"],
          "MultiMonitorSupport": false,
          "OverrideAccentColor": true,
          "CustomAccentColor": "#123456FF",
          "Keybindings": {
            "CreateHorizontalPanel": "Activation H",
            "ToggleFloatingMode": "F",
            "SwapLeft": "LeftShift+Left"
          }
        }
        "##;

        let migrated = Settings::from_legacy_json(input).unwrap();
        assert_eq!(
            migrated.settings.hotkeys.activation_key.as_deref(),
            Some("Shift+Win")
        );
        assert_eq!(
            migrated
                .settings
                .hotkeys
                .bindings
                .get("split-horizontal")
                .map(String::as_str),
            Some("H")
        );
        assert_eq!(
            migrated
                .settings
                .hotkeys
                .bindings
                .get("float")
                .map(String::as_str),
            Some("F")
        );
        assert_eq!(migrated.settings.behavior.window_padding, 8);
        assert!(!migrated.settings.display.multi_monitor);
        assert!(migrated.settings.theme.override_accent_color);
        assert_eq!(migrated.settings.theme.custom_accent_color, "#123456FF");
    }

    #[test]
    fn legacy_old_keybinding_shapes_migrate() {
        let input = r#"
        {
          "Keybindings": {
            "CreateVerticalPanel": { "Keys": ["V"], "IsDirectMode": false },
            "MoveFocusLeft": ["Left"]
          }
        }
        "#;

        let migrated = Settings::from_legacy_json(input).unwrap();
        assert_eq!(
            migrated
                .settings
                .hotkeys
                .bindings
                .get("split-vertical")
                .map(String::as_str),
            Some("V")
        );
        assert_eq!(
            migrated
                .settings
                .hotkeys
                .bindings
                .get("move-focus-left")
                .map(String::as_str),
            Some("Left")
        );
    }
}
