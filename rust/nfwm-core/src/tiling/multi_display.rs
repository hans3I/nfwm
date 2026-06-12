//! Multi-display manager: a facade over per-display tiling services.
//!
//! This mirrors the legacy `MultiDisplayTilingService` which delegates
//! tiling actions to one `TilingService` per display.

use crate::tiling::service::{PlacementResult, TilingService};
use crate::traits::PlacementProvider;
use crate::traits::WindowProvider;
use crate::types::{DisplayId, Rectangle, WindowId};
use std::collections::HashMap;

/// Per-display tiling state.
#[derive(Debug)]
pub struct DisplayState {
    pub display_id: DisplayId,
    pub work_area: Rectangle,
    pub service: TilingService,
}

/// Multi-display manager: manages one TilingService per display.
///
/// In single-display mode, this behaves like a single TilingService.
/// In multi-display mode, it routes commands to the correct display.
#[derive(Debug)]
pub struct MultiDisplayManager {
    displays: HashMap<DisplayId, DisplayState>,
    /// Whether multi-display mode is enabled.
    multi_monitor: bool,
    /// The primary display ID.
    primary: DisplayId,
}

impl MultiDisplayManager {
    pub fn new() -> Self {
        Self {
            displays: HashMap::new(),
            multi_monitor: false,
            primary: DisplayId::default(),
        }
    }

    /// Enable or disable multi-monitor mode.
    pub fn set_multi_monitor(&mut self, enabled: bool) {
        self.multi_monitor = enabled;
    }

    /// Check if multi-monitor mode is enabled.
    pub fn is_multi_monitor(&self) -> bool {
        self.multi_monitor
    }

    /// Add or update a display.
    pub fn add_display(
        &mut self,
        display_id: DisplayId,
        work_area: Rectangle,
    ) -> &mut DisplayState {
        let state = self
            .displays
            .entry(display_id)
            .or_insert_with(|| DisplayState {
                display_id,
                work_area,
                service: TilingService::with_work_area(work_area),
            });
        state.work_area = work_area;
        state.display_id = display_id;
        state
    }

    /// Remove a display.
    pub fn remove_display(&mut self, display_id: DisplayId) {
        self.displays.remove(&display_id);
    }

    /// Get the service for a display.
    pub fn get(&self, display_id: DisplayId) -> Option<&DisplayState> {
        self.displays.get(&display_id)
    }

    /// Get the service for a display mutably.
    pub fn get_mut(&mut self, display_id: DisplayId) -> Option<&mut DisplayState> {
        self.displays.get_mut(&display_id)
    }

    /// Get the primary display service.
    pub fn primary(&self) -> Option<&DisplayState> {
        self.displays.get(&self.primary)
    }

    /// Get the primary display service mutably.
    pub fn primary_mut(&mut self) -> Option<&mut DisplayState> {
        self.displays.get_mut(&self.primary)
    }

    /// Set the primary display.
    pub fn set_primary(&mut self, display_id: DisplayId) {
        self.primary = display_id;
    }

    /// Get all display IDs.
    pub fn display_ids(&self) -> Vec<DisplayId> {
        self.displays.keys().copied().collect()
    }

    /// Get the display that should manage a window based on its position.
    pub fn display_for_window(
        &self,
        provider: &dyn WindowProvider,
        window_id: WindowId,
    ) -> Option<DisplayId> {
        provider.display_id(window_id)
    }

    /// Discover windows and assign them to the correct display.
    ///
    /// Returns a map of display_id -> newly_managed windows.
    pub fn discover(
        &mut self,
        provider: &dyn WindowProvider,
        windows: &[WindowId],
    ) -> HashMap<DisplayId, Vec<WindowId>> {
        let mut result: HashMap<DisplayId, Vec<WindowId>> = HashMap::new();
        for &id in windows {
            let display_id = self
                .display_for_window(provider, id)
                .unwrap_or(self.primary);
            if let Some(state) = self.get_mut(display_id) {
                let newly = state.service.discover(provider, &[id]);
                if !newly.is_empty() {
                    result.entry(display_id).or_default().extend(newly);
                }
            }
        }
        result
    }

    /// Apply layout to all displays.
    pub fn apply_layout(
        &self,
        placement: &dyn PlacementProvider,
        shadow: bool,
    ) -> Vec<PlacementResult> {
        let mut results = Vec::new();
        for state in self.displays.values() {
            results.extend(state.service.apply_layout(placement, shadow));
        }
        results
    }

    /// Refresh layout on all displays.
    pub fn refresh(&mut self) {
        for state in self.displays.values_mut() {
            state.service.refresh();
        }
    }

    /// Start tiling on all displays.
    pub fn start(&mut self) {
        for state in self.displays.values_mut() {
            state.service.start();
        }
    }

    /// Stop tiling on all displays.
    pub fn stop(&mut self) {
        for state in self.displays.values_mut() {
            state.service.stop();
        }
    }

    /// Get the active display (primary in single-mode, or current).
    pub fn active_display(&self) -> Option<DisplayId> {
        if self.multi_monitor {
            // In multi-monitor mode, we need to track which display has focus.
            // For now, return the primary.
            Some(self.primary)
        } else {
            Some(self.primary)
        }
    }

    /// Get the active tiling service.
    pub fn active_service(&self) -> Option<&TilingService> {
        self.active_display()
            .and_then(|id| self.get(id))
            .map(|s| &s.service)
    }

    /// Get the active tiling service mutably.
    pub fn active_service_mut(&mut self) -> Option<&mut TilingService> {
        self.active_display()
            .and_then(|id| self.get_mut(id))
            .map(|s| &mut s.service)
    }
}

impl Default for MultiDisplayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::fake::*;
    use crate::types::{ProcessId, Size, VirtualDesktopId};

    fn make_provider(display_id: DisplayId) -> (FakeWindowProvider, WindowId) {
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
                display_id,
                virtual_desktop_id: VirtualDesktopId(0),
                focused: false,
            },
        );
        provider.focused = id;
        (provider, id)
    }

    #[test]
    fn manager_add_display() {
        let mut mgr = MultiDisplayManager::new();
        let id = DisplayId(1);
        mgr.add_display(id, Rectangle::new(0, 0, 1920, 1080));
        assert!(mgr.get(id).is_some());
    }

    #[test]
    fn manager_single_mode() {
        let mut mgr = MultiDisplayManager::new();
        mgr.set_multi_monitor(false);
        let id = DisplayId(1);
        mgr.add_display(id, Rectangle::new(0, 0, 1920, 1080));
        mgr.set_primary(id);
        assert_eq!(mgr.active_display(), Some(id));
    }

    #[test]
    fn manager_discover() {
        let mut mgr = MultiDisplayManager::new();
        let id = DisplayId(1);
        mgr.add_display(id, Rectangle::new(0, 0, 1920, 1080));
        mgr.set_primary(id);
        let (provider, window_id) = make_provider(id);
        let result = mgr.discover(&provider, &[window_id]);
        assert!(result.contains_key(&id));
        assert!(mgr
            .get(id)
            .unwrap()
            .service
            .workspace()
            .is_managed(window_id));
    }

    #[test]
    fn manager_start_stop() {
        let mut mgr = MultiDisplayManager::new();
        let id = DisplayId(1);
        mgr.add_display(id, Rectangle::new(0, 0, 1920, 1080));
        mgr.start();
        assert!(mgr.get(id).unwrap().service.is_active());
        mgr.stop();
        assert!(!mgr.get(id).unwrap().service.is_active());
    }

    #[test]
    fn manager_apply_layout() {
        let mut mgr = MultiDisplayManager::new();
        let id = DisplayId(1);
        mgr.add_display(id, Rectangle::new(0, 0, 1920, 1080));
        mgr.set_primary(id);
        let (provider, window_id) = make_provider(id);
        mgr.discover(&provider, &[window_id]);
        mgr.get_mut(id).unwrap().service.refresh();

        let fake = crate::traits::fake_placement::FakePlacementProviderMut::new();
        let results = mgr.apply_layout(&fake, false);
        assert!(!results.is_empty());
        assert!(fake.get(window_id).is_some());
    }

    #[test]
    fn manager_remove_display() {
        let mut mgr = MultiDisplayManager::new();
        let id = DisplayId(1);
        mgr.add_display(id, Rectangle::new(0, 0, 1920, 1080));
        mgr.remove_display(id);
        assert!(mgr.get(id).is_none());
    }

    #[test]
    fn manager_display_for_window() {
        let mgr = MultiDisplayManager::new();
        let (provider, window_id) = make_provider(DisplayId(0));
        let display = mgr.display_for_window(&provider, window_id);
        assert_eq!(display, Some(DisplayId(0)));
    }
}
