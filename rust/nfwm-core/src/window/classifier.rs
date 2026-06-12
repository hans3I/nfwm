//! Window classification: decides which windows should be tiled,
//! floated, or ignored based on their properties and configurable rules.

use super::registry::WindowEntry;

/// A classification rule for windows.
///
/// Rules are evaluated in order. The first matching rule wins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassificationRule {
    /// Match a window class name (exact or substring).
    ClassName { pattern: String, action: RuleAction },
    /// Match a window title (exact or substring).
    Title { pattern: String, action: RuleAction },
    /// Match a process name or executable (exact or substring).
    ProcessName { pattern: String, action: RuleAction },
    /// Match if the window is topmost.
    Topmost { action: RuleAction },
    /// Match if the window is not resizable.
    NonResizable { action: RuleAction },
    /// Match if the window is minimized or maximized.
    MinimizedOrMaximized { action: RuleAction },
}

/// The action to take when a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleAction {
    /// Float the window (do not tile, but track it).
    Float,
    /// Ignore the window (do not track at all).
    Ignore,
}

/// The classification result for a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Window should be tiled.
    Tiled,
    /// Window should be floated (excluded from tiling but tracked).
    Floating,
    /// Window should be ignored (not tracked).
    Ignored,
}

/// A classifier that evaluates windows against a list of rules.
///
/// Rules are ordered; the first matching rule wins. If no rule matches,
/// the default heuristic is applied.
#[derive(Debug, Clone, Default)]
pub struct WindowClassifier {
    rules: Vec<ClassificationRule>,
}

impl WindowClassifier {
    /// Create a new classifier with no rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a classifier with the default built-in rules.
    ///
    /// These rules match common system windows that should be ignored.
    pub fn with_defaults() -> Self {
        let mut classifier = Self::new();
        classifier.add_default_rules();
        classifier
    }

    /// Add a rule to the classifier.
    pub fn add_rule(&mut self, rule: ClassificationRule) {
        self.rules.push(rule);
    }

    /// Add the built-in default rules.
    pub fn add_default_rules(&mut self) {
        // Common system windows to ignore
        self.rules.push(ClassificationRule::ClassName {
            pattern: "Shell_TrayWnd".to_string(),
            action: RuleAction::Ignore,
        });
        self.rules.push(ClassificationRule::ClassName {
            pattern: "Progman".to_string(),
            action: RuleAction::Ignore,
        });
        self.rules.push(ClassificationRule::ClassName {
            pattern: "WorkerW".to_string(),
            action: RuleAction::Ignore,
        });
        self.rules.push(ClassificationRule::ClassName {
            pattern: "Windows.UI.Core.CoreWindow".to_string(),
            action: RuleAction::Ignore,
        });
        self.rules.push(ClassificationRule::ClassName {
            pattern: "StartMenuExperienceHost".to_string(),
            action: RuleAction::Ignore,
        });
        self.rules.push(ClassificationRule::ClassName {
            pattern: "SearchHost".to_string(),
            action: RuleAction::Ignore,
        });
        self.rules.push(ClassificationRule::ClassName {
            pattern: "Shell_SecondaryTrayWnd".to_string(),
            action: RuleAction::Ignore,
        });
        self.rules.push(ClassificationRule::ClassName {
            pattern: "ApplicationFrameWindow".to_string(),
            action: RuleAction::Ignore,
        });
    }

    /// Classify a window entry.
    ///
    /// Rules are evaluated first, then fallback heuristics.
    pub fn classify(&self, entry: &WindowEntry) -> WindowState {
        // Evaluate rules in order
        for rule in &self.rules {
            if Self::matches(rule, entry) {
                return match rule.action() {
                    RuleAction::Float => WindowState::Floating,
                    RuleAction::Ignore => WindowState::Ignored,
                };
            }
        }

        // Fallback heuristic
        if entry.minimized || entry.maximized {
            return WindowState::Floating;
        }
        if entry.topmost {
            return WindowState::Floating;
        }
        if !entry.resizable {
            return WindowState::Floating;
        }
        if entry.title.is_empty() && entry.class_name.is_empty() {
            return WindowState::Ignored;
        }

        WindowState::Tiled
    }

    fn matches(rule: &ClassificationRule, entry: &WindowEntry) -> bool {
        match rule {
            ClassificationRule::ClassName { pattern, .. } => entry.class_name.contains(pattern),
            ClassificationRule::Title { pattern, .. } => entry.title.contains(pattern),
            ClassificationRule::ProcessName { pattern, .. } => {
                // Process name matching is best-effort; we match against the numeric ID
                // as a string for simplicity in core. Real matching happens in Win32 layer.
                pattern
                    .parse::<u32>()
                    .is_ok_and(|pid| entry.process_id == pid)
                    || pattern.is_empty()
            }
            ClassificationRule::Topmost { .. } => entry.topmost,
            ClassificationRule::NonResizable { .. } => !entry.resizable,
            ClassificationRule::MinimizedOrMaximized { .. } => entry.minimized || entry.maximized,
        }
    }
}

impl ClassificationRule {
    fn action(&self) -> RuleAction {
        match self {
            ClassificationRule::ClassName { action, .. } => *action,
            ClassificationRule::Title { action, .. } => *action,
            ClassificationRule::ProcessName { action, .. } => *action,
            ClassificationRule::Topmost { action, .. } => *action,
            ClassificationRule::NonResizable { action, .. } => *action,
            ClassificationRule::MinimizedOrMaximized { action, .. } => *action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DisplayId, Rectangle, VirtualDesktopId, WindowId};

    fn make_entry(
        title: &str,
        class_name: &str,
        process_id: u32,
        minimized: bool,
        maximized: bool,
        topmost: bool,
        resizable: bool,
    ) -> WindowEntry {
        WindowEntry {
            id: WindowId(1),
            title: title.to_string(),
            class_name: class_name.to_string(),
            process_id,
            bounds: Rectangle::new(0, 0, 800, 600),
            visible: true,
            minimized,
            maximized,
            topmost,
            resizable,
            state: WindowState::Tiled,
            original_bounds: Some(Rectangle::new(0, 0, 800, 600)),
            display_id: Some(DisplayId(0)),
            virtual_desktop_id: Some(VirtualDesktopId(0)),
        }
    }

    #[test]
    fn classifier_default_ignores_system_classes() {
        let classifier = WindowClassifier::with_defaults();

        let entry = make_entry("", "Shell_TrayWnd", 0, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);

        let entry = make_entry("", "Progman", 0, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);

        let entry = make_entry("", "StartMenuExperienceHost", 0, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);
    }

    #[test]
    fn classifier_default_floats_minimized() {
        let classifier = WindowClassifier::with_defaults();
        let entry = make_entry("Notepad", "Notepad", 0, true, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Floating);
    }

    #[test]
    fn classifier_default_floats_maximized() {
        let classifier = WindowClassifier::with_defaults();
        let entry = make_entry("Notepad", "Notepad", 0, false, true, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Floating);
    }

    #[test]
    fn classifier_default_floats_topmost() {
        let classifier = WindowClassifier::with_defaults();
        let entry = make_entry("OnTop", "Notepad", 0, false, false, true, true);
        assert_eq!(classifier.classify(&entry), WindowState::Floating);
    }

    #[test]
    fn classifier_default_floats_non_resizable() {
        let classifier = WindowClassifier::with_defaults();
        let entry = make_entry("Fixed", "Notepad", 0, false, false, false, false);
        assert_eq!(classifier.classify(&entry), WindowState::Floating);
    }

    #[test]
    fn classifier_default_tiles_normal_window() {
        let classifier = WindowClassifier::with_defaults();
        let entry = make_entry("Notepad", "Notepad", 0, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Tiled);
    }

    #[test]
    fn classifier_custom_rules_take_priority() {
        let mut classifier = WindowClassifier::new();
        classifier.add_rule(ClassificationRule::Title {
            pattern: "Ignore Me".to_string(),
            action: RuleAction::Ignore,
        });
        let entry = make_entry("Ignore Me", "Notepad", 0, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);
    }

    #[test]
    fn classifier_rules_evaluated_in_order() {
        let mut classifier = WindowClassifier::new();
        classifier.add_rule(ClassificationRule::Title {
            pattern: "Float".to_string(),
            action: RuleAction::Float,
        });
        classifier.add_rule(ClassificationRule::ClassName {
            pattern: "Notepad".to_string(),
            action: RuleAction::Ignore,
        });
        // First rule wins: title "Float" should float
        let entry = make_entry("Float", "Notepad", 0, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Floating);
    }

    #[test]
    fn classifier_ignores_empty_window() {
        let classifier = WindowClassifier::with_defaults();
        let entry = make_entry("", "", 0, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);
    }

    #[test]
    fn classifier_custom_class_rule() {
        let mut classifier = WindowClassifier::new();
        classifier.add_rule(ClassificationRule::ClassName {
            pattern: "TaskManager".to_string(),
            action: RuleAction::Float,
        });
        let entry = make_entry("Task Manager", "TaskManager", 0, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Floating);
    }

    #[test]
    fn classifier_process_name_match() {
        let mut classifier = WindowClassifier::new();
        classifier.add_rule(ClassificationRule::ProcessName {
            pattern: "1234".to_string(),
            action: RuleAction::Ignore,
        });
        let entry = make_entry("Test", "Test", 1234, false, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);
    }

    #[test]
    fn classifier_topmost_rule() {
        let mut classifier = WindowClassifier::new();
        classifier.add_rule(ClassificationRule::Topmost {
            action: RuleAction::Ignore,
        });
        let entry = make_entry("Top", "Top", 0, false, false, true, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);
    }

    #[test]
    fn classifier_non_resizable_rule() {
        let mut classifier = WindowClassifier::new();
        classifier.add_rule(ClassificationRule::NonResizable {
            action: RuleAction::Ignore,
        });
        let entry = make_entry("Fixed", "Fixed", 0, false, false, false, false);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);
    }

    #[test]
    fn classifier_minimized_maximized_rule() {
        let mut classifier = WindowClassifier::new();
        classifier.add_rule(ClassificationRule::MinimizedOrMaximized {
            action: RuleAction::Ignore,
        });
        let entry = make_entry("Min", "Min", 0, true, false, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);
        let entry = make_entry("Max", "Max", 0, false, true, false, true);
        assert_eq!(classifier.classify(&entry), WindowState::Ignored);
    }
}
