//! Discovery service: polls the OS for windows and updates the registry.

use crate::traits::WindowProvider;
use crate::types::WindowId;
use crate::window::classifier::WindowClassifier;
use crate::window::registry::WindowRegistry;

/// Service that discovers windows and keeps the registry up to date.
///
/// This is a pure-logic component that does not make Win32 calls directly.
/// It requires a `WindowProvider` to enumerate windows.
#[derive(Debug)]
pub struct DiscoveryService {
    registry: WindowRegistry,
    classifier: WindowClassifier,
}

impl DiscoveryService {
    /// Create a new discovery service with default classification rules.
    pub fn new() -> Self {
        Self {
            registry: WindowRegistry::new(),
            classifier: WindowClassifier::with_defaults(),
        }
    }

    /// Create a new discovery service with a custom classifier.
    pub fn with_classifier(classifier: WindowClassifier) -> Self {
        Self {
            registry: WindowRegistry::new(),
            classifier,
        }
    }

    /// Refresh the registry with the current set of windows from the provider.
    ///
    /// Returns the list of newly discovered windows and the list of windows
    /// that were removed (no longer present).
    pub fn refresh(
        &mut self,
        provider: &dyn WindowProvider,
        enumerate: &dyn Fn() -> Vec<WindowId>,
    ) -> (Vec<WindowId>, Vec<WindowId>) {
        let current_ids = enumerate();
        let previous_ids = self.registry.ids();

        // Find removed windows
        let removed: Vec<WindowId> = previous_ids
            .iter()
            .filter(|id| !current_ids.contains(id))
            .copied()
            .collect();

        for id in &removed {
            self.registry.remove(*id);
        }

        // Find new windows
        let mut newly_discovered = Vec::new();
        for id in &current_ids {
            let is_new = !previous_ids.contains(id);
            let entry = self.registry.upsert(*id, provider);
            let state = self.classifier.classify(&entry);
            if let Some(e) = self.registry.get_mut(*id) {
                e.state = state;
            }
            if is_new {
                newly_discovered.push(*id);
            }
        }

        // Update focus
        if let Some(focused) = current_ids.iter().find(|id| provider.is_focused(**id)) {
            self.registry.set_focused(*focused);
        }

        (newly_discovered, removed)
    }

    /// Access the registry.
    pub fn registry(&self) -> &WindowRegistry {
        &self.registry
    }

    /// Access the registry mutably.
    pub fn registry_mut(&mut self) -> &mut WindowRegistry {
        &mut self.registry
    }

    /// Access the classifier.
    pub fn classifier(&self) -> &WindowClassifier {
        &self.classifier
    }

    /// Access the classifier mutably.
    pub fn classifier_mut(&mut self) -> &mut WindowClassifier {
        &mut self.classifier
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::fake::*;
    use crate::types::{DisplayId, ProcessId, Rectangle, Size, VirtualDesktopId, WindowId};
    use crate::window::classifier::WindowState;

    fn make_provider() -> (FakeWindowProvider, WindowId) {
        let mut provider = FakeWindowProvider::new();
        let id = WindowId(1);
        provider.add(
            id,
            FakeWindow {
                title: "Test".to_string(),
                class_name: "Notepad".to_string(),
                process_id: ProcessId(1234),
                bounds: Rectangle::new(0, 0, 800, 600),
                visible: true,
                minimized: false,
                maximized: false,
                topmost: false,
                resizable: true,
                min_size: Some(Size::new(200, 100)),
                display_id: DisplayId(0),
                virtual_desktop_id: VirtualDesktopId(0),
                focused: false,
            },
        );
        provider.focused = id;
        (provider, id)
    }

    #[test]
    fn discovery_refresh_discovers_window() {
        let (provider, id) = make_provider();
        let mut service = DiscoveryService::new();

        let (new, removed) = service.refresh(&provider, &|| vec![id]);
        assert_eq!(new, vec![id]);
        assert!(removed.is_empty());
        assert_eq!(service.registry().len(), 1);
        assert_eq!(service.registry().focused(), Some(id));
    }

    #[test]
    fn discovery_refresh_removes_closed_window() {
        let (provider, id) = make_provider();
        let mut service = DiscoveryService::new();

        // First discovery
        service.refresh(&provider, &|| vec![id]);
        assert_eq!(service.registry().len(), 1);

        // Second discovery: window is gone
        let (new, removed) = service.refresh(&provider, &|| vec![]);
        assert!(new.is_empty());
        assert_eq!(removed, vec![id]);
        assert_eq!(service.registry().len(), 0);
    }

    #[test]
    fn discovery_refresh_classifies_topmost_as_floating() {
        let mut provider = FakeWindowProvider::new();
        let id = WindowId(1);
        provider.add(
            id,
            FakeWindow {
                title: "Top".to_string(),
                class_name: "Notepad".to_string(),
                process_id: ProcessId(1234),
                bounds: Rectangle::new(0, 0, 800, 600),
                visible: true,
                minimized: false,
                maximized: false,
                topmost: true,
                resizable: true,
                min_size: Some(Size::new(200, 100)),
                display_id: DisplayId(0),
                virtual_desktop_id: VirtualDesktopId(0),
                focused: false,
            },
        );
        provider.focused = id;

        let mut service = DiscoveryService::new();
        service.refresh(&provider, &|| vec![id]);

        let entry = service.registry().get(id).unwrap();
        assert_eq!(entry.state, WindowState::Floating);
    }

    #[test]
    fn discovery_refresh_classifies_system_class_as_ignored() {
        let mut provider = FakeWindowProvider::new();
        let id = WindowId(1);
        provider.add(
            id,
            FakeWindow {
                title: "".to_string(),
                class_name: "Shell_TrayWnd".to_string(),
                process_id: ProcessId(0),
                bounds: Rectangle::new(0, 0, 1920, 40),
                visible: true,
                minimized: false,
                maximized: false,
                topmost: false,
                resizable: true,
                min_size: None,
                display_id: DisplayId(0),
                virtual_desktop_id: VirtualDesktopId(0),
                focused: false,
            },
        );

        let mut service = DiscoveryService::new();
        service.refresh(&provider, &|| vec![id]);

        let entry = service.registry().get(id).unwrap();
        assert_eq!(entry.state, WindowState::Ignored);
    }

    #[test]
    fn discovery_refresh_does_not_rediscover_existing() {
        let (provider, id) = make_provider();
        let mut service = DiscoveryService::new();

        service.refresh(&provider, &|| vec![id]);
        let (new, removed) = service.refresh(&provider, &|| vec![id]);
        assert!(new.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn discovery_refresh_preserves_original_bounds() {
        let (provider, id) = make_provider();
        let mut service = DiscoveryService::new();

        service.refresh(&provider, &|| vec![id]);
        let original = service.registry().get(id).unwrap().original_bounds;

        // Simulate window moved
        let mut provider2 = provider.clone();
        provider2.windows.get_mut(&id).unwrap().bounds = Rectangle::new(10, 10, 400, 300);

        service.refresh(&provider2, &|| vec![id]);
        let entry = service.registry().get(id).unwrap();
        assert_eq!(entry.original_bounds, original);
        assert_eq!(entry.bounds, Rectangle::new(10, 10, 400, 300));
    }
}
