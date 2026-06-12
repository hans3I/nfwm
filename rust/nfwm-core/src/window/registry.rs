//! Window registry: tracks discovered windows, their original positions,
//! and their classification for tiling.

use crate::traits::WindowProvider;
use crate::types::{DisplayId, Rectangle, VirtualDesktopId, WindowId};
use std::collections::HashMap;

use super::classifier::WindowState;

/// Metadata stored for each tracked window.
#[derive(Debug, Clone)]
pub struct WindowEntry {
    pub id: WindowId,
    pub title: String,
    pub class_name: String,
    pub process_id: u32,
    pub bounds: Rectangle,
    pub visible: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub topmost: bool,
    pub resizable: bool,
    pub state: WindowState,
    /// The original position before tiling started.
    pub original_bounds: Option<Rectangle>,
    /// The display the window was on when discovered.
    pub display_id: Option<DisplayId>,
    /// The virtual desktop the window was on when discovered.
    pub virtual_desktop_id: Option<VirtualDesktopId>,
}

/// The window registry: central store for all discovered windows.
///
/// The registry does not perform Win32 calls; it stores snapshots from
/// a `WindowProvider` and is updated by a `DiscoveryService`.
#[derive(Debug, Default)]
pub struct WindowRegistry {
    windows: HashMap<WindowId, WindowEntry>,
    /// Window IDs that are currently focused, in most-recent-first order.
    focus_history: Vec<WindowId>,
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or update a window from a provider snapshot.
    ///
    /// If the window is new, its original bounds are recorded.
    /// If the window already exists, only runtime fields are updated.
    pub fn upsert(&mut self, id: WindowId, provider: &dyn WindowProvider) -> WindowEntry {
        let title = provider.title(id).unwrap_or_default();
        let class_name = provider.class_name(id).unwrap_or_default();
        let process_id = provider.process_id(id).map(|p| p.0).unwrap_or(0);
        let bounds = provider.bounds(id).unwrap_or_default();
        let visible = provider.is_visible(id);
        let minimized = provider.is_minimized(id);
        let maximized = provider.is_maximized(id);
        let topmost = provider.is_topmost(id);
        let resizable = provider.is_resizable(id);
        let display_id = provider.display_id(id);
        let virtual_desktop_id = provider.virtual_desktop_id(id);

        let entry = self.windows.get_mut(&id);
        if let Some(entry) = entry {
            // Update existing: preserve original bounds
            entry.title = title;
            entry.class_name = class_name;
            entry.process_id = process_id;
            entry.bounds = bounds;
            entry.visible = visible;
            entry.minimized = minimized;
            entry.maximized = maximized;
            entry.topmost = topmost;
            entry.resizable = resizable;
            entry.display_id = display_id;
            entry.virtual_desktop_id = virtual_desktop_id;
            entry.clone()
        } else {
            // New window: record original bounds
            let entry = WindowEntry {
                id,
                title,
                class_name,
                process_id,
                bounds,
                visible,
                minimized,
                maximized,
                topmost,
                resizable,
                state: WindowState::Tiled,
                original_bounds: Some(bounds),
                display_id,
                virtual_desktop_id,
            };
            self.windows.insert(id, entry.clone());
            entry
        }
    }

    /// Remove a window from the registry.
    pub fn remove(&mut self, id: WindowId) -> Option<WindowEntry> {
        self.focus_history.retain(|&w| w != id);
        self.windows.remove(&id)
    }

    /// Get a window entry by ID.
    pub fn get(&self, id: WindowId) -> Option<&WindowEntry> {
        self.windows.get(&id)
    }

    /// Get a mutable window entry by ID.
    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut WindowEntry> {
        self.windows.get_mut(&id)
    }

    /// Mark a window as focused.
    pub fn set_focused(&mut self, id: WindowId) {
        self.focus_history.retain(|&w| w != id);
        self.focus_history.insert(0, id);
    }

    /// Get the currently focused window.
    pub fn focused(&self) -> Option<WindowId> {
        self.focus_history.first().copied()
    }

    /// Get the previous focused window.
    pub fn previous_focused(&self) -> Option<WindowId> {
        self.focus_history.get(1).copied()
    }

    /// Get all registered windows.
    pub fn all(&self) -> Vec<&WindowEntry> {
        self.windows.values().collect()
    }

    /// Get all windows matching a state.
    pub fn by_state(&self, state: WindowState) -> Vec<&WindowEntry> {
        self.windows.values().filter(|w| w.state == state).collect()
    }

    /// Get the count of registered windows.
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Clear the registry.
    pub fn clear(&mut self) {
        self.windows.clear();
        self.focus_history.clear();
    }

    /// Get all window IDs.
    pub fn ids(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::classifier::WindowClassifier;
    use super::super::registry::*;
    use crate::traits::fake::*;
    use crate::types::{DisplayId, ProcessId, Rectangle, Size, VirtualDesktopId, WindowId};

    #[test]
    fn registry_upsert_new_window() {
        let mut registry = WindowRegistry::new();
        let mut provider = FakeWindowProvider::new();
        let id = WindowId(1);
        provider.add(
            id,
            FakeWindow {
                title: "Test".to_string(),
                class_name: "TestClass".to_string(),
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

        let entry = registry.upsert(id, &provider);
        assert_eq!(entry.title, "Test");
        assert_eq!(entry.original_bounds, Some(Rectangle::new(0, 0, 800, 600)));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn registry_upsert_existing_preserves_original_bounds() {
        let mut registry = WindowRegistry::new();
        let mut provider = FakeWindowProvider::new();
        let id = WindowId(1);
        provider.add(
            id,
            FakeWindow {
                title: "Test".to_string(),
                class_name: "TestClass".to_string(),
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

        registry.upsert(id, &provider);

        // Simulate window moved
        provider.windows.get_mut(&id).unwrap().bounds = Rectangle::new(10, 10, 400, 300);

        let entry = registry.upsert(id, &provider);
        assert_eq!(entry.bounds, Rectangle::new(10, 10, 400, 300));
        assert_eq!(entry.original_bounds, Some(Rectangle::new(0, 0, 800, 600)));
    }

    #[test]
    fn registry_remove() {
        let mut registry = WindowRegistry::new();
        let mut provider = FakeWindowProvider::new();
        let id = WindowId(1);
        provider.add(
            id,
            FakeWindow {
                title: "Test".to_string(),
                class_name: "TestClass".to_string(),
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

        registry.upsert(id, &provider);
        registry.set_focused(id);
        assert_eq!(registry.focused(), Some(id));

        registry.remove(id);
        assert_eq!(registry.len(), 0);
        assert_eq!(registry.focused(), None);
    }

    #[test]
    fn registry_focus_history() {
        let mut registry = WindowRegistry::new();
        let mut provider = FakeWindowProvider::new();
        let id1 = WindowId(1);
        let id2 = WindowId(2);
        provider.add(
            id1,
            FakeWindow {
                title: "A".to_string(),
                class_name: "A".to_string(),
                process_id: ProcessId(1),
                bounds: Rectangle::new(0, 0, 100, 100),
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
        provider.add(
            id2,
            FakeWindow {
                title: "B".to_string(),
                class_name: "B".to_string(),
                process_id: ProcessId(2),
                bounds: Rectangle::new(0, 0, 100, 100),
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

        registry.upsert(id1, &provider);
        registry.upsert(id2, &provider);
        registry.set_focused(id1);
        registry.set_focused(id2);
        assert_eq!(registry.focused(), Some(id2));
        assert_eq!(registry.previous_focused(), Some(id1));

        registry.set_focused(id1);
        assert_eq!(registry.focused(), Some(id1));
        assert_eq!(registry.previous_focused(), Some(id2));
    }

    #[test]
    fn registry_by_state() {
        let mut registry = WindowRegistry::new();
        let mut provider = FakeWindowProvider::new();
        let id1 = WindowId(1);
        let id2 = WindowId(2);
        provider.add(
            id1,
            FakeWindow {
                title: "A".to_string(),
                class_name: "A".to_string(),
                process_id: ProcessId(1),
                bounds: Rectangle::new(0, 0, 100, 100),
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
        provider.add(
            id2,
            FakeWindow {
                title: "B".to_string(),
                class_name: "B".to_string(),
                process_id: ProcessId(2),
                bounds: Rectangle::new(0, 0, 100, 100),
                visible: true,
                minimized: false,
                maximized: false,
                topmost: true,
                resizable: true,
                min_size: None,
                display_id: DisplayId(0),
                virtual_desktop_id: VirtualDesktopId(0),
                focused: false,
            },
        );

        let classifier = WindowClassifier::default();
        let entry1 = registry.upsert(id1, &provider);
        let entry2 = registry.upsert(id2, &provider);
        registry.get_mut(id1).unwrap().state = classifier.classify(&entry1);
        registry.get_mut(id2).unwrap().state = classifier.classify(&entry2);

        assert_eq!(registry.by_state(WindowState::Tiled).len(), 1);
        assert_eq!(registry.by_state(WindowState::Floating).len(), 1);
        assert_eq!(registry.by_state(WindowState::Ignored).len(), 0);
    }
}
