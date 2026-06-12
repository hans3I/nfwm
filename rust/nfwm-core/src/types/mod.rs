//! Core domain types for nfwm.
//!
//! These types define the geometry, identifiers, and orientation used
//! throughout the tiling system. They are pure data with no OS dependencies.

pub mod geometry;
pub mod identifiers;

pub use geometry::*;
pub use identifiers::*;

/// Direction for focus or window movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Orientation for split panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelOrientation {
    Horizontal,
    Vertical,
}

/// A unique identifier for a tiling node within a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NodeId(pub u64);

/// Constraints for a node during the measure pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Constraints {
    pub min_width: i32,
    pub min_height: i32,
    pub max_width: i32,
    pub max_height: i32,
}

impl Constraints {
    pub fn new(min_width: i32, min_height: i32, max_width: i32, max_height: i32) -> Self {
        Self {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }

    pub fn unconstrained() -> Self {
        Self {
            min_width: 0,
            min_height: 0,
            max_width: i32::MAX,
            max_height: i32::MAX,
        }
    }
}

/// Result of a layout operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutResult {
    Ok,
    Unsatisfiable,
    MinSizeViolation,
}
