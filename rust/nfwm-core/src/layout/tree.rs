//! Tiling tree: root desktop tree with panel and window nodes.
//!
//! The tree uses an arena (Vec) to store all nodes, indexed by NodeId.
//! This avoids reference cycles and allows stable node references.

use std::collections::HashMap;

use crate::layout::node::{NodeType, TilingNode};
use crate::layout::panel::PanelNode;
use crate::types::{NodeId, PanelOrientation, Rectangle, Size, WindowId};

/// The root tiling tree for a desktop.
#[derive(Debug, Clone)]
pub struct DesktopTree {
    /// Arena of all nodes.
    nodes: Vec<TilingNode>,
    /// Panel data (children, flex constraints).
    panels: HashMap<NodeId, PanelNode>,
    /// Window ID to node ID mapping.
    window_map: HashMap<WindowId, NodeId>,
    /// Root panel node ID.
    root: Option<NodeId>,
    /// Work area for this desktop.
    pub work_area: Rectangle,
    /// Next node ID counter.
    next_id: u64,
}

impl DesktopTree {
    pub fn new(work_area: Rectangle) -> Self {
        Self {
            nodes: Vec::new(),
            panels: HashMap::new(),
            window_map: HashMap::new(),
            root: None,
            work_area,
            next_id: 1,
        }
    }

    /// Generate a new unique node ID.
    fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&TilingNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut TilingNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Get a panel by ID.
    pub fn get_panel(&self, id: NodeId) -> Option<&PanelNode> {
        self.panels.get(&id)
    }

    /// Get a mutable panel by ID.
    pub fn get_panel_mut(&mut self, id: NodeId) -> Option<&mut PanelNode> {
        self.panels.get_mut(&id)
    }

    /// Find a window node by window ID.
    pub fn find_window(&self, window_id: WindowId) -> Option<&TilingNode> {
        self.window_map
            .get(&window_id)
            .and_then(|&id| self.get_node(id))
    }

    /// Find the node ID for a window.
    pub fn find_window_id(&self, window_id: WindowId) -> Option<NodeId> {
        self.window_map.get(&window_id).copied()
    }

    /// Register a window in the lookup map.
    pub fn register_window(&mut self, window_id: WindowId, node_id: NodeId) {
        self.window_map.insert(window_id, node_id);
    }

    /// Unregister a window from the lookup map.
    pub fn unregister_window(&mut self, window_id: WindowId) {
        self.window_map.remove(&window_id);
    }

    /// Create a new window node and add it to the tree.
    pub fn create_window(&mut self, window_id: WindowId) -> NodeId {
        let id = self.next_id();
        let node = TilingNode::window(id, window_id);
        self.nodes.push(node);
        self.window_map.insert(window_id, id);
        id
    }

    /// Create a new split panel and add it to the tree.
    pub fn create_split(&mut self, orientation: PanelOrientation) -> NodeId {
        let id = self.next_id();
        let panel = PanelNode::new_split(id, orientation);
        self.nodes.push(panel.node.clone());
        self.panels.insert(id, panel);
        id
    }

    /// Create a new stack panel and add it to the tree.
    pub fn create_stack(&mut self) -> NodeId {
        let id = self.next_id();
        let panel = PanelNode::new_stack(id);
        self.nodes.push(panel.node.clone());
        self.panels.insert(id, panel);
        id
    }

    /// Create a new placeholder node.
    pub fn create_placeholder(&mut self) -> NodeId {
        let id = self.next_id();
        let node = TilingNode::placeholder(id);
        self.nodes.push(node);
        id
    }

    /// Set the root panel.
    pub fn set_root(&mut self, root_id: NodeId) {
        self.root = Some(root_id);
    }

    /// Get the root node ID.
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Attach a child node to a panel.
    pub fn attach(&mut self, panel_id: NodeId, index: usize, child_id: NodeId) {
        if let Some(child) = self.get_node_mut(child_id) {
            child.parent = Some(panel_id);
        }
        if let Some(panel) = self.get_panel_mut(panel_id) {
            panel.children.insert(index, child_id);
        }
        if let Some(window_id) = self.get_node(child_id).and_then(|n| n.window_id) {
            self.register_window(window_id, child_id);
        }
    }

    /// Detach a child node from a panel.
    pub fn detach(&mut self, panel_id: NodeId, child_id: NodeId) {
        if let Some(panel) = self.get_panel_mut(panel_id) {
            if let Some(idx) = panel.index_of(child_id) {
                panel.children.remove(idx);
            }
        }
        if let Some(child) = self.get_node_mut(child_id) {
            child.parent = None;
        }
        if let Some(window_id) = self.get_node(child_id).and_then(|n| n.window_id) {
            self.unregister_window(window_id);
        }
    }

    /// Replace a child at the given index in a panel.
    pub fn replace_child(
        &mut self,
        panel_id: NodeId,
        index: usize,
        new_child_id: NodeId,
    ) -> NodeId {
        let old = self
            .get_panel(panel_id)
            .map(|p| p.children[index])
            .unwrap_or(NodeId(0));
        if let Some(old_node) = self.get_node_mut(old) {
            old_node.parent = None;
        }
        if let Some(new_node) = self.get_node_mut(new_child_id) {
            new_node.parent = Some(panel_id);
        }
        if let Some(panel) = self.get_panel_mut(panel_id) {
            panel.children[index] = new_child_id;
        }
        if let Some(window_id) = self.get_node(old).and_then(|n| n.window_id) {
            self.unregister_window(window_id);
        }
        if let Some(window_id) = self.get_node(new_child_id).and_then(|n| n.window_id) {
            self.register_window(window_id, new_child_id);
        }
        old
    }

    /// Move a child from one index to another in a panel.
    pub fn move_child(&mut self, panel_id: NodeId, from: usize, to: usize) {
        if let Some(panel) = self.get_panel_mut(panel_id) {
            panel.move_child(from, to);
        }
    }

    /// Remove placeholder children from a panel.
    pub fn remove_placeholders(&mut self, panel_id: NodeId) {
        let to_remove: Vec<NodeId> = self
            .get_panel(panel_id)
            .map(|p| {
                p.children
                    .iter()
                    .filter(|&&id| {
                        self.get_node(id)
                            .map(|n| n.is_placeholder())
                            .unwrap_or(false)
                    })
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        for id in to_remove {
            self.detach(panel_id, id);
        }
    }

    /// Remove a panel if it has no children.
    pub fn remove_if_empty(&mut self, panel_id: NodeId) {
        let is_empty = self
            .get_panel(panel_id)
            .map(|p| p.children.is_empty())
            .unwrap_or(false);
        if is_empty {
            let parent_id = self.get_node(panel_id).and_then(|n| n.parent);
            if let Some(parent_id) = parent_id {
                self.detach(parent_id, panel_id);
            }
        }
    }

    /// Collapse a panel if it has only one child (replace self with child).
    pub fn collapse_if_single(&mut self, panel_id: NodeId) {
        let child_id = self.get_panel(panel_id).and_then(|p| {
            if p.children.len() == 1 {
                Some(p.children[0])
            } else {
                None
            }
        });
        if let Some(child_id) = child_id {
            let parent_id = self.get_node(panel_id).and_then(|n| n.parent);
            if let Some(parent_id) = parent_id {
                let idx = self.get_panel(parent_id).and_then(|p| p.index_of(panel_id));
                if let Some(idx) = idx {
                    self.replace_child(parent_id, idx, child_id);
                }
            }
        }
    }

    /// Clean up a panel after removal.
    pub fn cleanup_panel(&mut self, panel_id: NodeId, collapse: bool) {
        self.remove_placeholders(panel_id);
        self.remove_if_empty(panel_id);
        if collapse {
            self.collapse_if_single(panel_id);
        }
    }

    /// Swap two nodes in the tree.
    pub fn swap_nodes(&mut self, a: NodeId, b: NodeId) {
        let parent_a = self.get_node(a).and_then(|n| n.parent);
        let parent_b = self.get_node(b).and_then(|n| n.parent);

        if let (Some(pa), Some(pb)) = (parent_a, parent_b) {
            let idx_a = self.get_panel(pa).and_then(|p| p.index_of(a));
            let idx_b = self.get_panel(pb).and_then(|p| p.index_of(b));

            if let (Some(ia), Some(ib)) = (idx_a, idx_b) {
                if let Some(panel_a) = self.get_panel_mut(pa) {
                    panel_a.children[ia] = b;
                }
                if let Some(panel_b) = self.get_panel_mut(pb) {
                    panel_b.children[ib] = a;
                }
                if let Some(node_a) = self.get_node_mut(a) {
                    node_a.parent = Some(pb);
                }
                if let Some(node_b) = self.get_node_mut(b) {
                    node_b.parent = Some(pa);
                }
            }
        }
    }

    /// Remove a node from its parent.
    pub fn remove_node(&mut self, node_id: NodeId) {
        if let Some(parent_id) = self.get_node(node_id).and_then(|n| n.parent) {
            self.detach(parent_id, node_id);
        } else {
            if self.root == Some(node_id) {
                self.root = None;
            }
            if let Some(window_id) = self.get_node(node_id).and_then(|n| n.window_id) {
                self.unregister_window(window_id);
            }
        }
    }

    /// Remove a window from the tree, even if it's the root.
    pub fn remove_window(&mut self, window_id: WindowId) {
        if let Some(node_id) = self.find_window_id(window_id) {
            self.remove_node(node_id);
        }
    }

    /// Embed a node inside a new panel.
    pub fn embed(&mut self, node_id: NodeId, panel_id: NodeId) {
        if let Some(parent_id) = self.get_node(node_id).and_then(|n| n.parent) {
            let idx = self.get_panel(parent_id).and_then(|p| p.index_of(node_id));
            if let Some(idx) = idx {
                self.replace_child(parent_id, idx, panel_id);
            }
        }
        self.attach(panel_id, 0, node_id);
    }

    /// Split a node into a panel with the given orientation.
    pub fn split_node(&mut self, node_id: NodeId, orientation: PanelOrientation) -> NodeId {
        let parent_id = self.get_node(node_id).and_then(|n| n.parent);
        let new_panel = self.create_split(orientation);

        if let Some(parent_id) = parent_id {
            let idx = self.get_panel(parent_id).and_then(|p| p.index_of(node_id));
            if let Some(idx) = idx {
                self.replace_child(parent_id, idx, new_panel);
            }
        } else {
            self.set_root(new_panel);
        }

        self.attach(new_panel, 0, node_id);
        new_panel
    }

    /// Measure all nodes bottom-up.
    pub fn measure(&mut self) {
        if let Some(root) = self.root {
            self.measure_node(root);
        }
    }

    /// Arrange all nodes top-down.
    pub fn arrange(&mut self) {
        if let Some(root) = self.root {
            let work_area = self.work_area;
            self.arrange_node(root, work_area);
        }
    }

    /// Measure a single node (recursively).
    fn measure_node(&mut self, node_id: NodeId) {
        let node_type = self.get_node(node_id).map(|n| n.node_type);
        match node_type {
            Some(NodeType::Window) => {
                self.measure_window(node_id);
            }
            Some(NodeType::Placeholder) => {
                self.measure_placeholder(node_id);
            }
            Some(NodeType::Split) => {
                self.measure_split(node_id);
            }
            Some(NodeType::Stack) => {
                self.measure_stack(node_id);
            }
            None => {}
        }
    }

    fn measure_window(&mut self, node_id: NodeId) {
        if let Some(node) = self.get_node_mut(node_id) {
            node.content_min_size = Size::new(node.padding.width, node.padding.height);
        }
    }

    fn measure_placeholder(&mut self, node_id: NodeId) {
        if let Some(node) = self.get_node_mut(node_id) {
            node.content_min_size = Size::default();
        }
    }

    fn measure_split(&mut self, panel_id: NodeId) {
        let child_ids: Vec<NodeId> = self
            .get_panel(panel_id)
            .map(|p| p.children.clone())
            .unwrap_or_default();

        for child_id in &child_ids {
            self.measure_node(*child_id);
        }

        let mut total_width = 0;
        let mut total_height = 0;
        let orientation = self.get_node(panel_id).and_then(|n| n.orientation);

        for child_id in &child_ids {
            let child_min = self
                .get_node(*child_id)
                .map(|n| n.min_size())
                .unwrap_or_default();
            match orientation {
                Some(PanelOrientation::Horizontal) => {
                    total_width += child_min.width;
                    total_height = total_height.max(child_min.height);
                }
                Some(PanelOrientation::Vertical) => {
                    total_height += child_min.height;
                    total_width = total_width.max(child_min.width);
                }
                None => {}
            }
        }

        // Add padding and spacing
        let (spacing, window_count) = self
            .get_panel(panel_id)
            .map(|p| {
                let window_count = child_ids
                    .iter()
                    .filter(|&&id| self.get_node(id).map(|n| n.is_window()).unwrap_or(false))
                    .count() as i32;
                (p.spacing, window_count)
            })
            .unwrap_or((0, 0));
        let spacing_total = (window_count + 1) * spacing / 2;

        if let Some(node) = self.get_node_mut(panel_id) {
            node.content_min_size = Size::new(
                total_width + spacing_total + node.padding.width,
                total_height + spacing_total + node.padding.height,
            );
        }
    }

    fn measure_stack(&mut self, panel_id: NodeId) {
        let child_ids: Vec<NodeId> = self
            .get_panel(panel_id)
            .map(|p| p.children.clone())
            .unwrap_or_default();

        for child_id in &child_ids {
            self.measure_node(*child_id);
        }

        let mut max_width = 0;
        let mut max_height = 0;

        for child_id in &child_ids {
            let child_min = self
                .get_node(*child_id)
                .map(|n| n.min_size())
                .unwrap_or_default();
            max_width = max_width.max(child_min.width);
            max_height = max_height.max(child_min.height);
        }

        if let Some(node) = self.get_node_mut(panel_id) {
            node.content_min_size = Size::new(
                max_width + node.padding.width,
                max_height + node.padding.height,
            );
        }
    }

    /// Arrange a single node (recursively).
    fn arrange_node(&mut self, node_id: NodeId, rect: Rectangle) {
        let node_type = self.get_node(node_id).map(|n| n.node_type);
        match node_type {
            Some(NodeType::Window) | Some(NodeType::Placeholder) => {
                if let Some(node) = self.get_node_mut(node_id) {
                    node.computed_rect = rect;
                }
            }
            Some(NodeType::Split) => {
                self.arrange_split(node_id, rect);
            }
            Some(NodeType::Stack) => {
                self.arrange_stack(node_id, rect);
            }
            None => {}
        }
    }

    fn arrange_split(&mut self, panel_id: NodeId, rect: Rectangle) {
        let child_ids: Vec<NodeId> = self
            .get_panel(panel_id)
            .map(|p| p.children.clone())
            .unwrap_or_default();

        let orientation = self.get_node(panel_id).and_then(|n| n.orientation);
        let spacing = self.get_panel(panel_id).map(|p| p.spacing).unwrap_or(0);

        // Set up flex constraints
        let container_size = match orientation {
            Some(PanelOrientation::Horizontal) => rect.width,
            Some(PanelOrientation::Vertical) => rect.height,
            None => 0,
        };

        let mut flex = match super::flex::Flex::new(container_size as f64) {
            Ok(f) => f,
            Err(_) => return,
        };

        for child_id in &child_ids {
            let child = self.get_node(*child_id).cloned();
            let min = match orientation {
                Some(PanelOrientation::Horizontal) => {
                    child.as_ref().map(|c| c.min_size().width).unwrap_or(0)
                }
                Some(PanelOrientation::Vertical) => {
                    child.as_ref().map(|c| c.min_size().height).unwrap_or(0)
                }
                None => 0,
            };
            let max = match orientation {
                Some(PanelOrientation::Horizontal) => child
                    .as_ref()
                    .map(|c| c.max_size().width)
                    .unwrap_or(i32::MAX),
                Some(PanelOrientation::Vertical) => child
                    .as_ref()
                    .map(|c| c.max_size().height)
                    .unwrap_or(i32::MAX),
                None => i32::MAX,
            };
            let _ = flex.insert(flex.len(), min as f64, max as f64);
        }

        // Arrange children
        let mut last_rect = Rectangle::new(rect.x, rect.y, 0, 0);
        for (i, &child_id) in child_ids.iter().enumerate() {
            let width = flex.get(i).map(|f| f.width as i32).unwrap_or(0);
            let child_rect = match orientation {
                Some(PanelOrientation::Horizontal) => {
                    let r =
                        Rectangle::new(last_rect.x + last_rect.width, rect.y, width, rect.height);
                    last_rect = r;
                    r
                }
                Some(PanelOrientation::Vertical) => {
                    let r =
                        Rectangle::new(rect.x, last_rect.y + last_rect.height, rect.width, width);
                    last_rect = r;
                    r
                }
                None => rect,
            };

            // Apply spacing for window nodes
            let is_window = self
                .get_node(child_id)
                .map(|n| n.is_window())
                .unwrap_or(false);
            let final_rect = if is_window && spacing > 0 {
                Rectangle::new(
                    child_rect.x + spacing / 2,
                    child_rect.y + spacing / 2,
                    child_rect.width - spacing,
                    child_rect.height - spacing,
                )
            } else {
                child_rect
            };

            self.arrange_node(child_id, final_rect);
        }

        if let Some(node) = self.get_node_mut(panel_id) {
            node.computed_rect = rect;
        }
    }

    fn arrange_stack(&mut self, panel_id: NodeId, rect: Rectangle) {
        let child_ids: Vec<NodeId> = self
            .get_panel(panel_id)
            .map(|p| p.children.clone())
            .unwrap_or_default();

        for child_id in child_ids {
            self.arrange_node(child_id, rect);
        }

        if let Some(node) = self.get_node_mut(panel_id) {
            node.computed_rect = rect;
        }
    }

    /// Get all window nodes in the tree.
    pub fn windows(&self) -> Vec<&TilingNode> {
        self.nodes.iter().filter(|n| n.is_window()).collect()
    }

    /// Get all window node IDs in the tree.
    pub fn window_ids(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| n.is_window())
            .map(|n| n.id)
            .collect()
    }

    /// Get a text representation of the tree for debugging.
    pub fn visualize(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("DesktopTree (work_area: {:?})\n", self.work_area));
        if let Some(root) = self.root {
            self.visualize_node(root, 0, &mut result);
        } else {
            result.push_str("  (no root)\n");
        }
        result
    }

    fn visualize_node(&self, node_id: NodeId, depth: usize, result: &mut String) {
        let indent = "  ".repeat(depth);
        if let Some(node) = self.get_node(node_id) {
            let type_str = match node.node_type {
                NodeType::Window => format!("Window({:?})", node.window_id),
                NodeType::Placeholder => "Placeholder".to_string(),
                NodeType::Split => format!("Split({:?})", node.orientation),
                NodeType::Stack => "Stack".to_string(),
            };
            result.push_str(&format!(
                "{}{}: {} rect={:?}\n",
                indent, node_id.0, type_str, node.computed_rect
            ));

            if let Some(panel) = self.get_panel(node_id) {
                for &child_id in &panel.children {
                    self.visualize_node(child_id, depth + 1, result);
                }
            }
        }
    }

    /// Get the maximum child size for a given child in a panel.
    pub fn max_child_size(&self, panel_id: NodeId, child_id: NodeId) -> Option<Size> {
        let child = self.get_node(child_id)?;
        let panel = self.get_panel(panel_id)?;
        let reclaim = panel
            .flex
            .as_ref()
            .map(|f| f.container_width() - f.min_width())?;
        let rect = child.computed_rect;
        let orientation = panel.node.orientation?;
        Some(match orientation {
            PanelOrientation::Horizontal => Size::new(rect.width + reclaim as i32, rect.height),
            PanelOrientation::Vertical => Size::new(rect.width, rect.height + reclaim as i32),
        })
    }

    /// Get the maximum size available for inserting a new node into a panel.
    pub fn max_size_for_insert(&self, panel_id: NodeId, child_id: NodeId) -> Option<Size> {
        let child = self.get_node(child_id)?;
        let panel = self.get_panel(panel_id)?;
        let spacing = if child.is_window() { panel.spacing } else { 0 };
        let content = panel.node.content_rect();
        let min = panel.flex.as_ref().map(|f| f.min_width())?;
        let orientation = panel.node.orientation?;
        Some(match orientation {
            PanelOrientation::Horizontal => {
                Size::new(content.width - min as i32 - spacing, content.height)
            }
            PanelOrientation::Vertical => {
                Size::new(content.width, content.height - min as i32 - spacing)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_create_window() {
        let mut tree = DesktopTree::new(Rectangle::new(0, 0, 1920, 1080));
        let id = tree.create_window(WindowId(42));
        assert!(tree.get_node(id).is_some());
        assert!(tree.find_window(WindowId(42)).is_some());
    }

    #[test]
    fn tree_create_split() {
        let mut tree = DesktopTree::new(Rectangle::new(0, 0, 1920, 1080));
        let id = tree.create_split(PanelOrientation::Horizontal);
        assert!(tree.get_node(id).is_some());
        assert!(tree.get_panel(id).is_some());
    }

    #[test]
    fn tree_split_and_arrange() {
        let mut tree = DesktopTree::new(Rectangle::new(0, 0, 1000, 600));
        let w1 = tree.create_window(WindowId(1));
        let w2 = tree.create_window(WindowId(2));
        let panel = tree.create_split(PanelOrientation::Horizontal);
        tree.set_root(panel);

        tree.attach(panel, 0, w1);
        tree.attach(panel, 1, w2);

        tree.measure();
        tree.arrange();

        let w1_rect = tree.get_node(w1).unwrap().computed_rect;
        let w2_rect = tree.get_node(w2).unwrap().computed_rect;

        assert!(w1_rect.width > 0);
        assert!(w2_rect.width > 0);
        assert_eq!(w1_rect.height, 600);
        assert_eq!(w2_rect.height, 600);
    }

    #[test]
    fn tree_stack_arrange() {
        let mut tree = DesktopTree::new(Rectangle::new(0, 0, 1000, 600));
        let w1 = tree.create_window(WindowId(1));
        let w2 = tree.create_window(WindowId(2));
        let panel = tree.create_stack();
        tree.set_root(panel);

        tree.attach(panel, 0, w1);
        tree.attach(panel, 1, w2);

        tree.measure();
        tree.arrange();

        let w1_rect = tree.get_node(w1).unwrap().computed_rect;
        let w2_rect = tree.get_node(w2).unwrap().computed_rect;

        // Both should have the same rectangle (stacked)
        assert_eq!(w1_rect, w2_rect);
        assert_eq!(w1_rect.width, 1000);
        assert_eq!(w1_rect.height, 600);
    }

    #[test]
    fn tree_visualize() {
        let mut tree = DesktopTree::new(Rectangle::new(0, 0, 1000, 600));
        let w1 = tree.create_window(WindowId(1));
        let w2 = tree.create_window(WindowId(2));
        let panel = tree.create_split(PanelOrientation::Horizontal);
        tree.set_root(panel);

        tree.attach(panel, 0, w1);
        tree.attach(panel, 1, w2);

        tree.measure();
        tree.arrange();

        let vis = tree.visualize();
        assert!(vis.contains("DesktopTree"));
        assert!(vis.contains("Window"));
        assert!(vis.contains("Split"));
    }

    #[test]
    fn tree_swap_nodes() {
        let mut tree = DesktopTree::new(Rectangle::new(0, 0, 1000, 600));
        let w1 = tree.create_window(WindowId(1));
        let w2 = tree.create_window(WindowId(2));
        let panel = tree.create_split(PanelOrientation::Horizontal);
        tree.set_root(panel);

        tree.attach(panel, 0, w1);
        tree.attach(panel, 1, w2);

        tree.swap_nodes(w1, w2);

        let panel = tree.get_panel(panel).unwrap();
        assert_eq!(panel.children[0], w2);
        assert_eq!(panel.children[1], w1);
    }

    #[test]
    fn tree_split_node() {
        let mut tree = DesktopTree::new(Rectangle::new(0, 0, 1000, 600));
        let w1 = tree.create_window(WindowId(1));
        tree.set_root(w1);

        let new_panel = tree.split_node(w1, PanelOrientation::Horizontal);
        assert!(tree.get_panel(new_panel).is_some());
        assert_eq!(tree.root(), Some(new_panel));
        let panel = tree.get_panel(new_panel).unwrap();
        assert_eq!(panel.children[0], w1);
    }

    #[test]
    fn tree_cleanup() {
        let mut tree = DesktopTree::new(Rectangle::new(0, 0, 1000, 600));
        let w1 = tree.create_window(WindowId(1));
        let ph = tree.create_placeholder();
        let panel = tree.create_split(PanelOrientation::Horizontal);
        tree.set_root(panel);

        tree.attach(panel, 0, w1);
        tree.attach(panel, 1, ph);

        tree.remove_placeholders(panel);
        let panel = tree.get_panel(panel).unwrap();
        assert_eq!(panel.children.len(), 1);
        assert!(panel.children.contains(&w1));
    }
}
