//! Panel: split and stack panel containers.
//!
//! PanelNode is a data container. All mutation operations that interact
//! with the tree happen in DesktopTree methods.

use crate::layout::node::TilingNode;
use crate::types::{NodeId, PanelOrientation};

/// A panel node that contains child node IDs.
#[derive(Debug, Clone)]
pub struct PanelNode {
    pub node: TilingNode,
    pub children: Vec<NodeId>,
    pub spacing: i32,
    /// For split panels: the flex constraint solver.
    pub flex: Option<super::flex::Flex>,
}

impl PanelNode {
    pub fn new_split(id: NodeId, orientation: PanelOrientation) -> Self {
        Self {
            node: TilingNode::split(id, orientation),
            children: Vec::new(),
            spacing: 0,
            flex: None,
        }
    }

    pub fn new_stack(id: NodeId) -> Self {
        Self {
            node: TilingNode::stack(id),
            children: Vec::new(),
            spacing: 0,
            flex: None,
        }
    }

    /// Index of a child node.
    pub fn index_of(&self, child_id: NodeId) -> Option<usize> {
        self.children.iter().position(|&id| id == child_id)
    }

    /// Move a child from one index to another.
    pub fn move_child(&mut self, from: usize, to: usize) {
        if from == to || from >= self.children.len() || to > self.children.len() {
            return;
        }
        let item = self.children.remove(from);
        self.children.insert(to, item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_new_split() {
        let panel = PanelNode::new_split(NodeId(1), PanelOrientation::Horizontal);
        assert_eq!(panel.node.node_type, super::super::node::NodeType::Split);
        assert_eq!(panel.node.orientation, Some(PanelOrientation::Horizontal));
        assert!(panel.children.is_empty());
    }

    #[test]
    fn panel_index_of() {
        let mut panel = PanelNode::new_split(NodeId(1), PanelOrientation::Horizontal);
        panel.children = vec![NodeId(2), NodeId(3), NodeId(4)];
        assert_eq!(panel.index_of(NodeId(3)), Some(1));
        assert_eq!(panel.index_of(NodeId(99)), None);
    }

    #[test]
    fn panel_move_child() {
        let mut panel = PanelNode::new_split(NodeId(1), PanelOrientation::Horizontal);
        panel.children = vec![NodeId(2), NodeId(3), NodeId(4)];
        panel.move_child(0, 2);
        assert_eq!(panel.children, vec![NodeId(3), NodeId(4), NodeId(2)]);
    }
}
