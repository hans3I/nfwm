//! Tiling service: the application-facing tiling operations.
//!
//! This is the Rust equivalent of the legacy `ITilingService`.
//! It is pure logic and uses only `WindowProvider` for queries.

use std::collections::HashMap;

use crate::commands::Action;
use crate::layout::DesktopTree;
use crate::tiling::error::{TilingError, TilingResult};
use crate::tiling::workspace::TilingWorkspace;
use crate::traits::WindowProvider;
use crate::types::{
    Direction, NodeId, PanelOrientation, Point, Rectangle, VirtualDesktopId, WindowId,
};
use crate::window::classifier::WindowClassifier;
use crate::window::registry::WindowRegistry;

/// A pending tiling intent for multi-step operations.
#[derive(Debug, Clone)]
pub struct PendingIntent {
    pub action: Action,
    pub source_node: NodeId,
}

/// The tiling service: executes commands and manages layout state.
///
/// This struct owns the `TilingWorkspace` (per-desktop trees) and
/// coordinates all tiling operations. It does not perform OS calls
/// directly; it uses `WindowProvider` for queries.
#[derive(Debug)]
pub struct TilingService {
    workspace: TilingWorkspace,
    classifier: WindowClassifier,
    /// NodeId -> WindowId mapping for the current desktop.
    node_to_window: HashMap<NodeId, WindowId>,
    /// Current pending intent (if any).
    pending_intent: Option<PendingIntent>,
    /// Global active flag.
    active: bool,
    /// The default work area for new desktops.
    default_work_area: Rectangle,
    /// Window registry for tracking discovered windows.
    registry: WindowRegistry,
    /// Auto-split count threshold.
    #[allow(dead_code)]
    auto_split_count: usize,
}

impl TilingService {
    pub fn new() -> Self {
        Self {
            workspace: TilingWorkspace::new(),
            classifier: WindowClassifier::with_defaults(),
            node_to_window: HashMap::new(),
            pending_intent: None,
            active: false,
            default_work_area: Rectangle::new(0, 0, 1920, 1080),
            registry: WindowRegistry::new(),
            auto_split_count: 100,
        }
    }

    pub fn with_work_area(work_area: Rectangle) -> Self {
        let mut s = Self::new();
        s.default_work_area = work_area;
        s
    }

    // --- State queries ---

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn active_desktop(&self) -> Option<VirtualDesktopId> {
        self.workspace
            .current()
            .map(|_| self.workspace.current_desktop())
    }

    pub fn current_desktop(&self) -> VirtualDesktopId {
        self.workspace.current_desktop()
    }

    pub fn set_current_desktop(&mut self, id: VirtualDesktopId) {
        self.workspace.set_current(id);
    }

    pub fn pending_intent(&self) -> Option<&PendingIntent> {
        self.pending_intent.as_ref()
    }

    pub fn clear_pending_intent(&mut self) {
        self.pending_intent = None;
    }

    // --- Start / Stop / Toggle ---

    pub fn start(&mut self) {
        self.active = true;
        let id = self.workspace.current_desktop();
        if let Some(state) = self.workspace.get_mut(id) {
            state.active = true;
        }
    }

    pub fn stop(&mut self) {
        self.active = false;
        let id = self.workspace.current_desktop();
        if let Some(state) = self.workspace.get_mut(id) {
            state.active = false;
        }
    }

    pub fn toggle(&mut self) -> bool {
        self.active = !self.active;
        let id = self.workspace.current_desktop();
        if let Some(state) = self.workspace.get_mut(id) {
            state.active = self.active;
        }
        self.active
    }

    // --- Discovery ---

    pub fn discover(
        &mut self,
        provider: &dyn WindowProvider,
        windows: &[WindowId],
    ) -> Vec<WindowId> {
        let mut newly_managed = Vec::new();
        for &id in windows {
            let entry = self.registry.upsert(id, provider);
            let state = self.classifier.classify(&entry);
            if let Some(e) = self.registry.get_mut(id) {
                e.state = state;
            }
            match state {
                crate::window::classifier::WindowState::Tiled => {
                    if !self.workspace.is_managed(id) {
                        self.add_window(id);
                        newly_managed.push(id);
                    }
                }
                crate::window::classifier::WindowState::Floating => {
                    if self.workspace.is_managed(id) {
                        self.workspace.float_window(id);
                    }
                }
                crate::window::classifier::WindowState::Ignored => {
                    if self.workspace.is_managed(id) {
                        self.workspace.unregister_window(id);
                    }
                }
            }
        }
        newly_managed
    }

    pub fn sync_window_set(&mut self, windows: &[WindowId]) -> Vec<WindowId> {
        let removed: Vec<WindowId> = self
            .registry
            .ids()
            .into_iter()
            .filter(|id| !windows.contains(id))
            .collect();

        for id in &removed {
            self.registry.remove(*id);
            self.workspace.unregister_window(*id);
        }

        removed
    }

    fn add_window(&mut self, window_id: WindowId) {
        let desktop_id = self.workspace.current_desktop();
        self.workspace.get_or_create(desktop_id, self.default_work_area);
        let node_id = self.workspace.register_window(desktop_id, window_id);
        self.node_to_window.insert(node_id, window_id);
        let tree = self.workspace.current_tree_mut().unwrap();
        if let Some(root) = tree.root() {
            if let Some(panel) = tree.get_panel(root) {
                // Root is already a panel, just attach
                tree.attach(root, panel.children.len(), node_id);
            } else {
                // Root is a window node, wrap it in a horizontal panel
                let new_panel = tree.create_split(PanelOrientation::Horizontal);
                let old_root = root;
                tree.set_root(new_panel);
                tree.attach(new_panel, 0, old_root);
                tree.attach(new_panel, 1, node_id);
            }
        } else {
            tree.set_root(node_id);
        }
    }

    pub fn refresh(&mut self) {
        if let Some(tree) = self.workspace.current_tree_mut() {
            tree.measure();
            tree.arrange();
        }
    }

    pub fn set_work_area(&mut self, work_area: Rectangle) {
        self.default_work_area = work_area;
        let desktop_id = self.workspace.current_desktop();
        self.workspace.update_work_area(desktop_id, work_area);
    }

    // --- Focus queries ---

    pub fn focused_window(&self) -> Option<WindowId> {
        self.workspace.focused_window()
    }

    pub fn focused_node(&self) -> Option<NodeId> {
        self.workspace.focused_node()
    }

    pub fn get_bounds(&self) -> Option<Rectangle> {
        self.workspace.current_tree().map(|t| t.work_area)
    }

    pub fn find_closest(&self, point: Point) -> Option<WindowId> {
        self.workspace.closest_window(point)
    }

    // --- Command validation ---

    fn require_active(&self) -> TilingResult<()> {
        if !self.active {
            return Err(TilingError::NotActive);
        }
        Ok(())
    }

    fn is_in_stack_panel(tree: &DesktopTree, node_id: NodeId) -> bool {
        let mut current = Some(node_id);
        while let Some(id) = current {
            if let Some(node) = tree.get_node(id) {
                if let Some(parent_id) = node.parent {
                    if let Some(parent) = tree.get_node(parent_id) {
                        if parent.node_type == crate::layout::node::NodeType::Stack {
                            return true;
                        }
                    }
                    current = Some(parent_id);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        false
    }

    fn find_adjacent(tree: &DesktopTree, node_id: NodeId, direction: Direction) -> Option<NodeId> {
        let node = tree.get_node(node_id)?;
        let parent_id = node.parent?;
        let parent = tree.get_panel(parent_id)?;
        let orientation = tree.get_node(parent_id)?.orientation?;
        let idx = parent.index_of(node_id)?;

        let sibling_idx = match (orientation, direction) {
            (PanelOrientation::Horizontal, Direction::Left) if idx > 0 => Some(idx - 1),
            (PanelOrientation::Horizontal, Direction::Right) if idx + 1 < parent.children.len() => {
                Some(idx + 1)
            }
            (PanelOrientation::Vertical, Direction::Up) if idx > 0 => Some(idx - 1),
            (PanelOrientation::Vertical, Direction::Down) if idx + 1 < parent.children.len() => {
                Some(idx + 1)
            }
            _ => None,
        };

        if let Some(sibling_idx) = sibling_idx {
            let sibling_id = parent.children[sibling_idx];
            return Self::find_leaf_in_direction(tree, sibling_id, direction);
        }

        let parent_node = tree.get_node(parent_id)?;
        if let Some(_grandparent_id) = parent_node.parent {
            Self::find_adjacent(tree, parent_id, direction)
        } else {
            None
        }
    }

    fn find_leaf_in_direction(
        tree: &DesktopTree,
        node_id: NodeId,
        direction: Direction,
    ) -> Option<NodeId> {
        let node = tree.get_node(node_id)?;
        if node.is_window() || node.is_placeholder() {
            return Some(node_id);
        }
        let panel = tree.get_panel(node_id)?;
        let orientation = tree.get_node(node_id)?.orientation?;
        let idx = match (orientation, direction) {
            (PanelOrientation::Horizontal, Direction::Right) => 0,
            (PanelOrientation::Horizontal, Direction::Left) => {
                panel.children.len().saturating_sub(1)
            }
            (PanelOrientation::Vertical, Direction::Down) => 0,
            (PanelOrientation::Vertical, Direction::Up) => panel.children.len().saturating_sub(1),
            _ => 0,
        };
        panel
            .children
            .get(idx)
            .and_then(|&id| Self::find_leaf_in_direction(tree, id, direction))
    }

    fn find_adjacent_for_swap(
        tree: &DesktopTree,
        focused: NodeId,
        direction: Direction,
    ) -> Option<NodeId> {
        let node = tree.get_node(focused)?;
        let parent_id = node.parent?;
        let parent = tree.get_panel(parent_id)?;
        let orientation = tree.get_node(parent_id)?.orientation?;
        let idx = parent.index_of(focused)?;

        let sibling_idx = match (orientation, direction) {
            (PanelOrientation::Horizontal, Direction::Left) if idx > 0 => Some(idx - 1),
            (PanelOrientation::Horizontal, Direction::Right) if idx + 1 < parent.children.len() => {
                Some(idx + 1)
            }
            (PanelOrientation::Vertical, Direction::Up) if idx > 0 => Some(idx - 1),
            (PanelOrientation::Vertical, Direction::Down) if idx + 1 < parent.children.len() => {
                Some(idx + 1)
            }
            _ => None,
        };
        sibling_idx.map(|i| parent.children[i])
    }

    pub fn can_move_focus(&self, direction: Direction) -> bool {
        if !self.active {
            return false;
        }
        let tree = match self.workspace.current_tree() {
            Some(t) => t,
            None => return false,
        };
        let focused = match self.workspace.focused_node() {
            Some(n) => n,
            None => return false,
        };
        Self::find_adjacent(tree, focused, direction).is_some()
    }

    pub fn move_focus(&mut self, direction: Direction) -> TilingResult<()> {
        self.require_active()?;
        let tree = match self.workspace.current_tree() {
            Some(t) => t,
            None => return Err(TilingError::Failed),
        };
        let focused = self
            .workspace
            .focused_node()
            .ok_or(TilingError::MissingTarget)?;
        let adjacent = Self::find_adjacent(tree, focused, direction)
            .ok_or(TilingError::MissingAdjacentWindow)?;
        self.workspace.set_focus(adjacent);
        Ok(())
    }

    pub fn can_swap(&self, direction: Direction) -> bool {
        if !self.active {
            return false;
        }
        let tree = match self.workspace.current_tree() {
            Some(t) => t,
            None => return false,
        };
        let focused = match self.workspace.focused_node() {
            Some(n) => n,
            None => return false,
        };
        Self::find_adjacent_for_swap(tree, focused, direction).is_some()
    }

    pub fn swap(&mut self, direction: Direction) -> TilingResult<()> {
        self.require_active()?;
        let focused = self
            .workspace
            .focused_node()
            .ok_or(TilingError::MissingTarget)?;
        let tree = self
            .workspace
            .current_tree_mut()
            .ok_or(TilingError::Failed)?;
        let adjacent = Self::find_adjacent_for_swap(tree, focused, direction)
            .ok_or(TilingError::MissingAdjacentWindow)?;
        tree.swap_nodes(focused, adjacent);
        self.refresh();
        Ok(())
    }

    pub fn can_move_window(&self, direction: Direction) -> bool {
        self.can_swap(direction)
    }

    pub fn move_window(&mut self, direction: Direction) -> TilingResult<()> {
        self.require_active()?;
        let focused = self
            .workspace
            .focused_node()
            .ok_or(TilingError::MissingTarget)?;
        let tree = self
            .workspace
            .current_tree_mut()
            .ok_or(TilingError::Failed)?;
        let adjacent = Self::find_adjacent_for_swap(tree, focused, direction)
            .ok_or(TilingError::MissingAdjacentWindow)?;

        let node = tree.get_node(focused).ok_or(TilingError::MissingTarget)?;
        let parent_a = node.parent.ok_or(TilingError::ModifiesTopLevelPanel)?;
        let adjacent_node = tree
            .get_node(adjacent)
            .ok_or(TilingError::MissingAdjacentWindow)?;
        let parent_b = adjacent_node
            .parent
            .ok_or(TilingError::MissingAdjacentWindow)?;

        if parent_a == parent_b {
            let parent = tree.get_panel(parent_a).ok_or(TilingError::Failed)?;
            let idx_a = parent.index_of(focused).ok_or(TilingError::Failed)?;
            let idx_b = parent.index_of(adjacent).ok_or(TilingError::Failed)?;
            tree.move_child(parent_a, idx_a, idx_b);
        } else {
            match direction {
                Direction::Left | Direction::Up => {
                    let idx = tree
                        .get_panel(parent_b)
                        .and_then(|p| p.index_of(adjacent))
                        .ok_or(TilingError::Failed)?;
                    tree.detach(parent_a, focused);
                    tree.attach(parent_b, idx + 1, focused);
                }
                Direction::Right | Direction::Down => {
                    let idx = tree
                        .get_panel(parent_b)
                        .and_then(|p| p.index_of(adjacent))
                        .ok_or(TilingError::Failed)?;
                    tree.detach(parent_a, focused);
                    tree.attach(parent_b, idx, focused);
                }
            }
        }

        self.refresh();
        Ok(())
    }

    pub fn can_split(&self, vertical: bool) -> bool {
        let _ = vertical;
        if !self.active {
            return false;
        }
        let tree = match self.workspace.current_tree() {
            Some(t) => t,
            None => return false,
        };
        let focused = match self.workspace.focused_node() {
            Some(n) => n,
            None => return false,
        };
        !Self::is_in_stack_panel(tree, focused)
    }

    pub fn split(&mut self, vertical: bool) -> TilingResult<()> {
        self.require_active()?;
        let focused = self
            .workspace
            .focused_node()
            .ok_or(TilingError::MissingTarget)?;
        let tree = self
            .workspace
            .current_tree_mut()
            .ok_or(TilingError::Failed)?;
        if Self::is_in_stack_panel(tree, focused) {
            return Err(TilingError::NestingInStackPanel);
        }
        let orientation = if vertical {
            PanelOrientation::Vertical
        } else {
            PanelOrientation::Horizontal
        };
        let _panel_id = tree.split_node(focused, orientation);
        self.refresh();
        Ok(())
    }

    pub fn can_stack(&self) -> bool {
        if !self.active {
            return false;
        }
        let tree = match self.workspace.current_tree() {
            Some(t) => t,
            None => return false,
        };
        let focused = match self.workspace.focused_node() {
            Some(n) => n,
            None => return false,
        };
        !Self::is_in_stack_panel(tree, focused)
    }

    pub fn stack(&mut self) -> TilingResult<()> {
        self.require_active()?;
        let focused = self
            .workspace
            .focused_node()
            .ok_or(TilingError::MissingTarget)?;
        let tree = self
            .workspace
            .current_tree_mut()
            .ok_or(TilingError::Failed)?;
        if Self::is_in_stack_panel(tree, focused) {
            return Err(TilingError::NestingInStackPanel);
        }
        let parent_id = tree
            .get_node(focused)
            .and_then(|n| n.parent)
            .ok_or(TilingError::ModifiesTopLevelPanel)?;
        let new_panel = tree.create_stack();
        let idx = tree
            .get_panel(parent_id)
            .and_then(|p| p.index_of(focused))
            .ok_or(TilingError::Failed)?;
        tree.replace_child(parent_id, idx, new_panel);
        tree.attach(new_panel, 0, focused);
        self.refresh();
        Ok(())
    }

    pub fn can_float(&self) -> bool {
        if !self.active {
            return false;
        }
        self.workspace.focused_window().is_some()
    }

    pub fn float_window(&mut self) -> TilingResult<()> {
        self.require_active()?;
        let window = self
            .workspace
            .focused_window()
            .ok_or(TilingError::MissingTarget)?;
        if self.workspace.is_floating(window) {
            return Err(TilingError::AlreadyFloating);
        }
        self.workspace.float_window(window);
        self.refresh();
        Ok(())
    }

    pub fn can_pull_up(&self) -> bool {
        if !self.active {
            return false;
        }
        let tree = match self.workspace.current_tree() {
            Some(t) => t,
            None => return false,
        };
        let focused = match self.workspace.focused_node() {
            Some(n) => n,
            None => return false,
        };
        let node = match tree.get_node(focused) {
            Some(n) => n,
            None => return false,
        };
        if let Some(parent_id) = node.parent {
            tree.root() != Some(parent_id)
        } else {
            false
        }
    }

    pub fn pull_up(&mut self) -> TilingResult<()> {
        self.require_active()?;
        let focused = self
            .workspace
            .focused_node()
            .ok_or(TilingError::MissingTarget)?;
        let tree = self
            .workspace
            .current_tree_mut()
            .ok_or(TilingError::Failed)?;
        let node = tree.get_node(focused).ok_or(TilingError::MissingTarget)?;
        let parent_id = node.parent.ok_or(TilingError::PullsBeyondTopLevelPanel)?;
        if tree.root() == Some(parent_id) {
            return Err(TilingError::PullsBeyondTopLevelPanel);
        }
        let grandparent_id = tree
            .get_node(parent_id)
            .and_then(|n| n.parent)
            .ok_or(TilingError::PullsBeyondTopLevelPanel)?;
        let parent_idx = tree
            .get_panel(grandparent_id)
            .and_then(|p| p.index_of(parent_id))
            .ok_or(TilingError::Failed)?;
        tree.detach(parent_id, focused);
        tree.attach(grandparent_id, parent_idx, focused);
        tree.cleanup_panel(parent_id, true);
        self.refresh();
        Ok(())
    }

    pub fn can_resize(&self, _orientation: PanelOrientation) -> bool {
        if !self.active {
            return false;
        }
        self.workspace.focused_node().is_some()
    }

    pub fn resize(&mut self, orientation: PanelOrientation, delta: i32) -> TilingResult<()> {
        self.require_active()?;
        let focused = self
            .workspace
            .focused_node()
            .ok_or(TilingError::MissingTarget)?;
        let tree = self
            .workspace
            .current_tree_mut()
            .ok_or(TilingError::Failed)?;
        let node = tree.get_node(focused).ok_or(TilingError::MissingTarget)?;
        let parent_id = node.parent.ok_or(TilingError::Failed)?;
        let panel = tree.get_panel(parent_id).ok_or(TilingError::Failed)?;
        let panel_orientation = tree
            .get_node(parent_id)
            .and_then(|n| n.orientation)
            .ok_or(TilingError::Failed)?;

        if panel_orientation != orientation {
            return Err(TilingError::InvalidTarget);
        }

        let idx = panel.index_of(focused).ok_or(TilingError::Failed)?;
        let _sibling_idx = if idx + 1 < panel.children.len() {
            idx + 1
        } else if idx > 0 {
            idx - 1
        } else {
            return Err(TilingError::MissingAdjacentWindow);
        };

        if let Some(node) = tree.get_node_mut(focused) {
            match orientation {
                PanelOrientation::Horizontal => {
                    node.padding.width += delta;
                    node.padding.width = node.padding.width.max(0);
                }
                PanelOrientation::Vertical => {
                    node.padding.height += delta;
                    node.padding.height = node.padding.height.max(0);
                }
            }
        }

        self.refresh();
        Ok(())
    }

    // --- Intent operations ---

    pub fn set_intent(&mut self, action: Action) -> TilingResult<()> {
        let focused = self
            .workspace
            .focused_node()
            .ok_or(TilingError::MissingTarget)?;
        self.pending_intent = Some(PendingIntent {
            action,
            source_node: focused,
        });
        Ok(())
    }

    // --- Registry access ---

    pub fn registry(&self) -> &WindowRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut WindowRegistry {
        &mut self.registry
    }

    pub fn workspace(&self) -> &TilingWorkspace {
        &self.workspace
    }

    pub fn workspace_mut(&mut self) -> &mut TilingWorkspace {
        &mut self.workspace
    }

    // --- Placement engine ---

    /// Apply the current layout to all tiled windows.
    ///
    /// If `shadow` is true, placements are logged but not applied.
    /// Returns a list of placement results for each window.
    pub fn apply_layout(
        &self,
        placement: &dyn crate::traits::PlacementProvider,
        shadow: bool,
    ) -> Vec<PlacementResult> {
        let mut results = Vec::new();
        if let Some(tree) = self.workspace.current_tree() {
            let work_area = tree.work_area;
            for node in tree.windows() {
                if let Some(window_id) = node.window_id {
                    let rect = node.computed_rect.clamp_within(work_area);
                    if rect.is_empty() {
                        continue;
                    }
                    if shadow {
                        results.push(PlacementResult {
                            window_id,
                            rect,
                            success: true,
                            error: None,
                        });
                    } else {
                        let result = placement.set_bounds(window_id, rect);
                        results.push(PlacementResult {
                            window_id,
                            rect,
                            success: result.is_ok(),
                            error: result.err().map(|e| e.to_string()),
                        });
                    }
                }
            }
        }
        results
    }

    /// Restore windows to their original bounds.
    ///
    /// Uses the registry's `original_bounds` for each window.
    pub fn restore_layout(
        &self,
        placement: &dyn crate::traits::PlacementProvider,
        windows: &[WindowId],
    ) -> Vec<PlacementResult> {
        let mut results = Vec::new();
        for &window_id in windows {
            if let Some(entry) = self.registry.get(window_id) {
                if let Some(original) = entry.original_bounds {
                    let result = placement.set_bounds(window_id, original);
                    results.push(PlacementResult {
                        window_id,
                        rect: original,
                        success: result.is_ok(),
                        error: result.err().map(|e| e.to_string()),
                    });
                }
            }
        }
        results
    }
}

/// Result of a single placement operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementResult {
    pub window_id: WindowId,
    pub rect: Rectangle,
    pub success: bool,
    pub error: Option<String>,
}

impl Default for TilingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::fake::*;
    use crate::types::{DisplayId, ProcessId, Size, VirtualDesktopId};

    fn setup_service() -> (TilingService, FakeWindowProvider, WindowId) {
        let mut service = TilingService::with_work_area(Rectangle::new(0, 0, 1000, 600));
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

        service.set_current_desktop(VirtualDesktopId(0));
        service.start();
        service.discover(&provider, &[id]);
        service
            .workspace
            .set_focus(service.workspace.find_node(id).unwrap());

        (service, provider, id)
    }

    fn add_second_window(
        service: &mut TilingService,
        provider: &mut FakeWindowProvider,
    ) -> WindowId {
        let id = WindowId(2);
        provider.add(
            id,
            FakeWindow {
                title: "Second".to_string(),
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
        service.discover(provider, &[id]);
        id
    }

    #[test]
    fn service_start_stop() {
        let mut service = TilingService::new();
        assert!(!service.is_active());
        service.start();
        assert!(service.is_active());
        service.stop();
        assert!(!service.is_active());
    }

    #[test]
    fn service_toggle() {
        let mut service = TilingService::new();
        assert!(service.toggle());
        assert!(!service.toggle());
    }

    #[test]
    fn service_discover_adds_window() {
        let (service, _provider, id) = setup_service();
        assert!(service.workspace.is_managed(id));
    }

    #[test]
    fn service_discover_removes_closed_window() {
        let (mut service, mut provider, id) = setup_service();
        let id2 = add_second_window(&mut service, &mut provider);
        assert!(service.workspace.is_managed(id2));

        provider.windows.remove(&id2);
        let removed = service.sync_window_set(&[id]);

        assert_eq!(removed, vec![id2]);
        assert!(!service.workspace.is_managed(id2));
    }

    #[test]
    fn service_uses_configured_work_area() {
        let (service, _provider, _id) = setup_service();
        assert_eq!(service.get_bounds(), Some(Rectangle::new(0, 0, 1000, 600)));
    }

    #[test]
    fn service_move_focus() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);

        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let children = tree.get_panel(root).unwrap().children.clone();
        service.workspace.set_focus(children[0]);
        assert!(service.can_move_focus(Direction::Right));
        service.move_focus(Direction::Right).unwrap();
        assert_eq!(service.focused_node(), Some(children[1]));
    }

    #[test]
    fn service_swap() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);

        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let children = tree.get_panel(root).unwrap().children.clone();
        service.workspace.set_focus(children[0]);
        assert!(service.can_swap(Direction::Right));
        service.swap(Direction::Right).unwrap();

        let tree = service.workspace.current_tree().unwrap();
        let panel = tree.get_panel(root).unwrap();
        assert_eq!(panel.children[0], children[1]);
        assert_eq!(panel.children[1], children[0]);
    }

    #[test]
    fn service_split() {
        let (mut service, _provider, _id) = setup_service();
        assert!(service.can_split(false));
        service.split(false).unwrap();
        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        assert!(tree.get_panel(root).is_some());
    }

    #[test]
    fn service_split_fails_in_stack() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);
        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let children = tree.get_panel(root).unwrap().children.clone();
        service.workspace.set_focus(children[0]);
        service.stack().unwrap();
        assert!(!service.can_split(false));
        assert_eq!(service.split(false), Err(TilingError::NestingInStackPanel));
    }

    #[test]
    fn service_stack() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);
        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let children = tree.get_panel(root).unwrap().children.clone();
        service.workspace.set_focus(children[0]);
        assert!(service.can_stack());
        service.stack().unwrap();
        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        assert!(tree.get_panel(root).is_some());
    }

    #[test]
    fn service_float() {
        let (mut service, _provider, id) = setup_service();
        assert!(service.can_float());
        service.float_window().unwrap();
        assert!(service.workspace.is_floating(id));
        assert!(!service.workspace.is_managed(id));
    }

    #[test]
    fn service_float_fails_when_not_active() {
        let mut service = TilingService::new();
        assert!(!service.can_float());
        assert_eq!(service.float_window(), Err(TilingError::NotActive));
    }

    #[test]
    fn service_pull_up() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);

        // Split the first window to create a nested panel
        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let children = tree.get_panel(root).unwrap().children.clone();
        service.workspace.set_focus(children[0]);
        service.split(false).unwrap();

        // Now tree has: Root(H) -> [Panel(H) -> [Win1], Win2]
        // Focus on Win1 (inside nested panel)
        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let panel = tree.get_panel(root).unwrap();
        let nested_panel = panel.children[0];
        let nested = tree.get_panel(nested_panel).unwrap();
        let win1 = nested.children[0];
        service.workspace.set_focus(win1);

        assert!(service.can_pull_up());
        service.pull_up().unwrap();
    }

    #[test]
    fn service_move_window() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);

        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let children = tree.get_panel(root).unwrap().children.clone();
        service.workspace.set_focus(children[0]);
        service.move_window(Direction::Right).unwrap();

        let tree = service.workspace.current_tree().unwrap();
        let panel = tree.get_panel(root).unwrap();
        assert_eq!(panel.children[0], children[1]);
        assert_eq!(panel.children[1], children[0]);
    }

    #[test]
    fn service_resize() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);

        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let children = tree.get_panel(root).unwrap().children.clone();
        service.workspace.set_focus(children[0]);
        assert!(service.can_resize(PanelOrientation::Horizontal));
        service.resize(PanelOrientation::Horizontal, 10).unwrap();
    }

    #[test]
    fn service_closest_window() {
        let (mut service, _provider, id) = setup_service();
        let tree = service.workspace.current_tree_mut().unwrap();
        let node_id = tree.find_window_id(id).unwrap();
        if let Some(node) = tree.get_node_mut(node_id) {
            node.computed_rect = Rectangle::new(100, 100, 200, 200);
        }
        let closest = service.find_closest(Point::new(150, 150));
        assert_eq!(closest, Some(id));
    }

    #[test]
    fn service_find_adjacent() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);

        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let children = tree.get_panel(root).unwrap().children.clone();
        service.workspace.set_focus(children[0]);
        assert!(service.can_move_focus(Direction::Right));
        let adjacent = service.move_focus(Direction::Right);
        assert!(adjacent.is_ok());
        assert_eq!(service.focused_node(), Some(children[1]));
    }

    #[test]
    fn service_refresh() {
        let (mut service, _provider, _id) = setup_service();
        service.refresh();
        let tree = service.workspace.current_tree().unwrap();
        let root = tree.root().unwrap();
        let node = tree.get_node(root).unwrap();
        assert!(!node.computed_rect.is_empty());
    }

    #[test]
    fn service_pending_intent() {
        let (mut service, _provider, _id) = setup_service();
        service.set_intent(Action::SplitHorizontal).unwrap();
        assert!(service.pending_intent().is_some());
        service.clear_pending_intent();
        assert!(service.pending_intent().is_none());
    }

    #[test]
    fn service_not_active_errors() {
        let mut service = TilingService::new();
        assert_eq!(service.split(false), Err(TilingError::NotActive));
        assert_eq!(service.stack(), Err(TilingError::NotActive));
        assert_eq!(
            service.move_focus(Direction::Left),
            Err(TilingError::NotActive)
        );
        assert_eq!(service.swap(Direction::Left), Err(TilingError::NotActive));
        assert_eq!(
            service.move_window(Direction::Left),
            Err(TilingError::NotActive)
        );
        assert_eq!(service.float_window(), Err(TilingError::NotActive));
        assert_eq!(service.pull_up(), Err(TilingError::NotActive));
        assert_eq!(
            service.resize(PanelOrientation::Horizontal, 10),
            Err(TilingError::NotActive)
        );
    }

    #[test]
    fn service_apply_layout() {
        let (mut service, mut provider, id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);
        service.refresh();

        let fake = crate::traits::fake_placement::FakePlacementProviderMut::new();
        let results = service.apply_layout(&fake, false);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
        assert!(fake.get(id).is_some());
    }

    #[test]
    fn service_apply_layout_shadow() {
        let (mut service, mut provider, _id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);
        service.refresh();

        let fake = crate::traits::fake_placement::FakePlacementProviderMut::new();
        let results = service.apply_layout(&fake, true);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
        // Shadow mode should not actually place
        assert_eq!(fake.placements.borrow().len(), 0);
    }

    #[test]
    fn service_apply_layout_clamps_to_work_area() {
        let (mut service, mut provider, id) = setup_service();
        let id2 = add_second_window(&mut service, &mut provider);
        service.set_work_area(Rectangle::new(0, 0, 400, 200));
        service.refresh();

        let fake = crate::traits::fake_placement::FakePlacementProviderMut::new();
        let results = service.apply_layout(&fake, false);

        assert_eq!(results.len(), 2);
        for window_id in [id, id2] {
            let rect = fake.get(window_id).unwrap();
            assert!(rect.x >= 0);
            assert!(rect.y >= 0);
            assert!(rect.x + rect.width <= 400);
            assert!(rect.y + rect.height <= 200);
        }
    }

    #[test]
    fn service_restore_layout() {
        let (mut service, mut provider, id) = setup_service();
        let _id2 = add_second_window(&mut service, &mut provider);

        let fake = crate::traits::fake_placement::FakePlacementProviderMut::new();
        let results = service.restore_layout(&fake, &[id]);
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].rect, Rectangle::new(0, 0, 800, 600));
    }
}
