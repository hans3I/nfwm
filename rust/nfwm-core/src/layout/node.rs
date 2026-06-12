//! Tiling node: base trait and node types for the layout tree.

use crate::types::{NodeId, Rectangle, Size, WindowId};

/// The type of a tiling node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Split,
    Stack,
    Window,
    Placeholder,
}

/// A tiling node in the layout tree.
///
/// Each node has a measure/arrange lifecycle:
/// 1. `measure()` computes minimum size bottom-up
/// 2. `arrange()` computes final rectangles top-down
#[derive(Debug, Clone)]
pub struct TilingNode {
    pub id: NodeId,
    pub node_type: NodeType,
    /// Minimum content size (without padding).
    pub content_min_size: Size,
    /// Maximum content size.
    pub content_max_size: Size,
    /// Padding around the content.
    pub padding: Rectangle,
    /// Computed border rectangle from arrange.
    pub computed_rect: Rectangle,
    /// Parent node ID (None for root).
    pub parent: Option<NodeId>,
    /// For Window nodes: the window reference.
    pub window_id: Option<WindowId>,
    /// For panel nodes: orientation.
    pub orientation: Option<crate::types::PanelOrientation>,
}

impl TilingNode {
    /// Create a new window node.
    pub fn window(id: NodeId, window_id: WindowId) -> Self {
        Self {
            id,
            node_type: NodeType::Window,
            content_min_size: Size::default(),
            content_max_size: Size::new(i32::MAX, i32::MAX),
            padding: Rectangle::default(),
            computed_rect: Rectangle::default(),
            parent: None,
            window_id: Some(window_id),
            orientation: None,
        }
    }

    /// Create a new placeholder node.
    pub fn placeholder(id: NodeId) -> Self {
        Self {
            id,
            node_type: NodeType::Placeholder,
            content_min_size: Size::default(),
            content_max_size: Size::new(i32::MAX, i32::MAX),
            padding: Rectangle::default(),
            computed_rect: Rectangle::default(),
            parent: None,
            window_id: None,
            orientation: None,
        }
    }

    /// Create a new split panel node.
    pub fn split(id: NodeId, orientation: crate::types::PanelOrientation) -> Self {
        Self {
            id,
            node_type: NodeType::Split,
            content_min_size: Size::default(),
            content_max_size: Size::new(i32::MAX, i32::MAX),
            padding: Rectangle::default(),
            computed_rect: Rectangle::default(),
            parent: None,
            window_id: None,
            orientation: Some(orientation),
        }
    }

    /// Create a new stack panel node.
    pub fn stack(id: NodeId) -> Self {
        Self {
            id,
            node_type: NodeType::Stack,
            content_min_size: Size::default(),
            content_max_size: Size::new(i32::MAX, i32::MAX),
            padding: Rectangle::default(),
            computed_rect: Rectangle::default(),
            parent: None,
            window_id: None,
            orientation: None,
        }
    }

    /// Get the minimum size including padding.
    pub fn min_size(&self) -> Size {
        Size::new(
            self.content_min_size.width + self.padding.width,
            self.content_min_size.height + self.padding.height,
        )
    }

    /// Get the maximum size including padding.
    pub fn max_size(&self) -> Size {
        Size::new(
            self.content_max_size.width + self.padding.width,
            self.content_max_size.height + self.padding.height,
        )
    }

    /// Get the content rectangle (computed_rect minus padding).
    pub fn content_rect(&self) -> Rectangle {
        Rectangle::new(
            self.computed_rect.x + self.padding.x,
            self.computed_rect.y + self.padding.y,
            self.computed_rect.width - self.padding.width,
            self.computed_rect.height - self.padding.height,
        )
    }

    /// Check if this is a panel (Split or Stack).
    pub fn is_panel(&self) -> bool {
        matches!(self.node_type, NodeType::Split | NodeType::Stack)
    }

    /// Check if this is a window node.
    pub fn is_window(&self) -> bool {
        matches!(self.node_type, NodeType::Window)
    }

    /// Check if this is a placeholder.
    pub fn is_placeholder(&self) -> bool {
        matches!(self.node_type, NodeType::Placeholder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PanelOrientation;

    #[test]
    fn node_window() {
        let node = TilingNode::window(NodeId(1), WindowId(42));
        assert!(node.is_window());
        assert_eq!(node.window_id, Some(WindowId(42)));
        assert!(!node.is_panel());
    }

    #[test]
    fn node_split() {
        let node = TilingNode::split(NodeId(1), PanelOrientation::Horizontal);
        assert!(node.is_panel());
        assert_eq!(node.node_type, NodeType::Split);
        assert_eq!(node.orientation, Some(PanelOrientation::Horizontal));
    }

    #[test]
    fn node_min_size_with_padding() {
        let mut node = TilingNode::window(NodeId(1), WindowId(42));
        node.content_min_size = Size::new(100, 50);
        node.padding = Rectangle::new(5, 5, 10, 10);
        assert_eq!(node.min_size(), Size::new(110, 60));
    }
}
