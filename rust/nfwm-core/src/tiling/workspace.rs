//! Tiling workspace: per-desktop state tracking.
//!
//! A `TilingWorkspace` owns one `DesktopTree` per virtual desktop,
//! tracks the focused node, and maps window IDs to nodes.

use std::collections::HashMap;

use crate::layout::DesktopTree;
use crate::types::{NodeId, Rectangle, VirtualDesktopId, WindowId};

/// State for a single virtual desktop.
#[derive(Debug, Clone)]
pub struct DesktopState {
    pub tree: DesktopTree,
    /// The currently focused node on this desktop.
    pub focused_node: Option<NodeId>,
    /// Windows that are floating on this desktop.
    pub floating: Vec<WindowId>,
    /// Whether tiling is active on this desktop.
    pub active: bool,
}

impl DesktopState {
    pub fn new(tree: DesktopTree) -> Self {
        Self {
            tree,
            focused_node: None,
            floating: Vec::new(),
            active: true,
        }
    }
}

/// The tiling workspace: manages per-desktop state.
#[derive(Debug)]
pub struct TilingWorkspace {
    desktops: HashMap<VirtualDesktopId, DesktopState>,
    /// The currently active virtual desktop.
    current_desktop: VirtualDesktopId,
    /// Windows that are globally ignored (not managed by any desktop).
    pub ignored: Vec<WindowId>,
}

impl TilingWorkspace {
    pub fn new() -> Self {
        Self {
            desktops: HashMap::new(),
            current_desktop: VirtualDesktopId::default(),
            ignored: Vec::new(),
        }
    }

    /// Get or create a desktop state.
    pub fn get_or_create(
        &mut self,
        id: VirtualDesktopId,
        work_area: Rectangle,
    ) -> &mut DesktopState {
        self.desktops
            .entry(id)
            .or_insert_with(|| DesktopState::new(DesktopTree::new(work_area)))
    }

    /// Get a desktop state.
    pub fn get(&self, id: VirtualDesktopId) -> Option<&DesktopState> {
        self.desktops.get(&id)
    }

    /// Get a desktop state mutably.
    pub fn get_mut(&mut self, id: VirtualDesktopId) -> Option<&mut DesktopState> {
        self.desktops.get_mut(&id)
    }

    /// Get the current desktop state.
    pub fn current(&self) -> Option<&DesktopState> {
        self.get(self.current_desktop)
    }

    /// Get the current desktop state mutably.
    pub fn current_mut(&mut self) -> Option<&mut DesktopState> {
        self.get_mut(self.current_desktop)
    }

    /// Set the current desktop.
    pub fn current_desktop(&self) -> VirtualDesktopId {
        self.current_desktop
    }

    pub fn set_current(&mut self, id: VirtualDesktopId) {
        self.current_desktop = id;
    }

    /// Remove a desktop.
    pub fn remove(&mut self, id: VirtualDesktopId) {
        self.desktops.remove(&id);
    }

    /// Get all desktop IDs.
    pub fn desktop_ids(&self) -> Vec<VirtualDesktopId> {
        self.desktops.keys().copied().collect()
    }

    /// Check if a window is managed by any desktop.
    pub fn is_managed(&self, window_id: WindowId) -> bool {
        self.desktops
            .values()
            .any(|d| d.tree.find_window(window_id).is_some())
    }

    /// Find the desktop that manages a window.
    pub fn find_desktop(&self, window_id: WindowId) -> Option<VirtualDesktopId> {
        self.desktops
            .iter()
            .find(|(_, d)| d.tree.find_window(window_id).is_some())
            .map(|(id, _)| *id)
    }

    /// Register a window on a desktop.
    pub fn register_window(&mut self, desktop_id: VirtualDesktopId, window_id: WindowId) -> NodeId {
        let state = self.get_or_create(desktop_id, Rectangle::new(0, 0, 1920, 1080));
        state.tree.create_window(window_id)
    }

    /// Unregister a window from all desktops.
    pub fn unregister_window(&mut self, window_id: WindowId) {
        for state in self.desktops.values_mut() {
            if let Some(node_id) = state.tree.find_window_id(window_id) {
                state.tree.remove_node(node_id);
            }
        }
    }

    /// Set the focused node on the current desktop.
    pub fn set_focus(&mut self, node_id: NodeId) {
        if let Some(state) = self.current_mut() {
            state.focused_node = Some(node_id);
        }
    }

    /// Get the focused node on the current desktop.
    pub fn focused_node(&self) -> Option<NodeId> {
        self.current().and_then(|d| d.focused_node)
    }

    /// Get the focused window on the current desktop.
    pub fn focused_window(&self) -> Option<WindowId> {
        let node_id = self.focused_node()?;
        self.current()
            .and_then(|d| d.tree.get_node(node_id))
            .and_then(|n| n.window_id)
    }

    /// Get the tree for the current desktop.
    pub fn current_tree(&self) -> Option<&DesktopTree> {
        self.current().map(|d| &d.tree)
    }

    /// Get the tree for the current desktop mutably.
    pub fn current_tree_mut(&mut self) -> Option<&mut DesktopTree> {
        self.current_mut().map(|d| &mut d.tree)
    }

    /// Get the node for a window on the current desktop.
    pub fn find_node(&self, window_id: WindowId) -> Option<NodeId> {
        self.current()
            .and_then(|d| d.tree.find_window_id(window_id))
    }

    /// Check if a window is floating on the current desktop.
    pub fn is_floating(&self, window_id: WindowId) -> bool {
        self.current()
            .map(|d| d.floating.contains(&window_id))
            .unwrap_or(false)
    }

    /// Float a window on the current desktop.
    pub fn float_window(&mut self, window_id: WindowId) {
        if let Some(state) = self.current_mut() {
            if !state.floating.contains(&window_id) {
                state.floating.push(window_id);
            }
            state.tree.remove_window(window_id);
        }
    }

    /// Unfloat a window on the current desktop.
    pub fn unfloat_window(&mut self, window_id: WindowId) {
        if let Some(state) = self.current_mut() {
            state.floating.retain(|&id| id != window_id);
        }
    }

    /// Toggle tiling for the current desktop.
    pub fn toggle_active(&mut self) -> bool {
        if let Some(state) = self.current_mut() {
            state.active = !state.active;
            state.active
        } else {
            false
        }
    }

    /// Get the closest window to a point on the current desktop.
    pub fn closest_window(&self, point: crate::types::Point) -> Option<WindowId> {
        let tree = self.current_tree()?;
        let mut closest = None;
        let mut closest_dist = i64::MAX;
        for window in tree.windows() {
            let rect = window.computed_rect;
            if !rect.is_empty() {
                let center_x = rect.x + rect.width / 2;
                let center_y = rect.y + rect.height / 2;
                let dx = (center_x - point.x) as i64;
                let dy = (center_y - point.y) as i64;
                let dist = dx * dx + dy * dy;
                if dist < closest_dist {
                    closest_dist = dist;
                    closest = window.window_id;
                }
            }
        }
        closest
    }
}

impl Default for TilingWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Point;

    #[test]
    fn workspace_create_desktop() {
        let mut ws = TilingWorkspace::new();
        let id = VirtualDesktopId(1);
        let state = ws.get_or_create(id, Rectangle::new(0, 0, 1920, 1080));
        assert!(state.active);
    }

    #[test]
    fn workspace_register_window() {
        let mut ws = TilingWorkspace::new();
        let id = VirtualDesktopId(1);
        ws.set_current(id);
        let node_id = ws.register_window(id, WindowId(42));
        assert!(ws.is_managed(WindowId(42)));
        assert_eq!(ws.find_node(WindowId(42)), Some(node_id));
    }

    #[test]
    fn workspace_focus() {
        let mut ws = TilingWorkspace::new();
        let id = VirtualDesktopId(1);
        let node_id = ws.register_window(id, WindowId(42));
        ws.set_current(id);
        ws.set_focus(node_id);
        assert_eq!(ws.focused_node(), Some(node_id));
        assert_eq!(ws.focused_window(), Some(WindowId(42)));
    }

    #[test]
    fn workspace_float() {
        let mut ws = TilingWorkspace::new();
        let id = VirtualDesktopId(1);
        ws.set_current(id);
        ws.register_window(id, WindowId(42));
        ws.float_window(WindowId(42));
        assert!(ws.is_floating(WindowId(42)));
        assert!(!ws.is_managed(WindowId(42)));
    }

    #[test]
    fn workspace_toggle_active() {
        let mut ws = TilingWorkspace::new();
        let id = VirtualDesktopId(1);
        ws.set_current(id);
        ws.get_or_create(id, Rectangle::new(0, 0, 1920, 1080));
        assert!(!ws.toggle_active()); // toggle off -> returns false
        assert!(ws.toggle_active()); // toggle on -> returns true
    }

    #[test]
    fn workspace_closest_window() {
        let mut ws = TilingWorkspace::new();
        let id = VirtualDesktopId(1);
        let node_id = ws.register_window(id, WindowId(1));
        let node_id2 = ws.register_window(id, WindowId(2));
        ws.set_current(id);
        ws.set_focus(node_id);
        ws.set_focus(node_id2);

        // Manually set computed_rects
        if let Some(tree) = ws.current_tree_mut() {
            if let Some(node) = tree.get_node_mut(node_id) {
                node.computed_rect = Rectangle::new(0, 0, 100, 100);
            }
            if let Some(node) = tree.get_node_mut(node_id2) {
                node.computed_rect = Rectangle::new(200, 200, 100, 100);
            }
        }

        let closest = ws.closest_window(Point::new(50, 50));
        assert_eq!(closest, Some(WindowId(1)));

        let closest = ws.closest_window(Point::new(250, 250));
        assert_eq!(closest, Some(WindowId(2)));
    }
}
