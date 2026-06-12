//! Flex: flexible sizing and constraint handling for split panels.
//!
//! This is a Rust port of the C# Flex constraint solver. It manages
//! a list of items with min/max width constraints, allocating space
//! proportionally while respecting bounds.

use thiserror::Error;

/// Error when flex constraints cannot be satisfied.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum FlexError {
    #[error("Constraints not satisfiable: {reason}")]
    Unsatisfiable { reason: String },
    #[error("Container width {width} is less than minimum {min}")]
    ContainerTooSmall { width: f64, min: f64 },
}

/// A single flex item with constraints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexItem {
    /// Current allocated width.
    pub width: f64,
    /// Minimum width.
    pub min_width: f64,
    /// Maximum width.
    pub max_width: f64,
}

impl FlexItem {
    pub fn new(min_width: f64, max_width: f64) -> Result<Self, FlexError> {
        if min_width < 0.0 {
            return Err(FlexError::Unsatisfiable {
                reason: "min_width must be >= 0".to_string(),
            });
        }
        if min_width > max_width {
            return Err(FlexError::Unsatisfiable {
                reason: "min_width > max_width".to_string(),
            });
        }
        if !max_width.is_finite() {
            return Err(FlexError::Unsatisfiable {
                reason: "max_width must be finite".to_string(),
            });
        }
        Ok(Self {
            width: min_width,
            min_width,
            max_width,
        })
    }

    /// Clamp width to [min_width, max_width].
    pub fn clamp(&self, width: f64) -> f64 {
        width.clamp(self.min_width, self.max_width)
    }
}

/// A flex container that distributes space among items.
#[derive(Debug, Clone, PartialEq)]
pub struct Flex {
    /// Total available width.
    container_width: f64,
    /// Items in the container.
    items: Vec<FlexItem>,
}

impl Flex {
    /// Create a new flex container with the given width.
    pub fn new(container_width: f64) -> Result<Self, FlexError> {
        if container_width < 1.0 {
            return Err(FlexError::Unsatisfiable {
                reason: format!("container_width ({container_width}) < 1"),
            });
        }
        Ok(Self {
            container_width,
            items: Vec::new(),
        })
    }

    /// Insert a new item at the given index.
    pub fn insert(
        &mut self,
        index: usize,
        min_width: f64,
        max_width: f64,
    ) -> Result<(), FlexError> {
        let item = FlexItem::new(min_width, max_width)?;

        // Check if we can fit the minimum width
        let current_min: f64 = self.items.iter().map(|i| i.min_width).sum();
        if current_min + item.min_width > self.container_width {
            return Err(FlexError::Unsatisfiable {
                reason: format!(
                    "cannot fit item with min_width {} into container (used {}, total {})",
                    item.min_width, current_min, self.container_width
                ),
            });
        }

        // Allocate space for the new item
        let new_width = self.allocate(item.min_width, item.max_width);

        // If new_width exceeds available space, reclaim from existing items
        if new_width > self.unused_width() {
            let reclaim = self.unused_width() - new_width;
            let leftover = self.apply_deltas(reclaim, None);
            if leftover.abs() > 0.001 {
                return Err(FlexError::Unsatisfiable {
                    reason: "could not reclaim enough space for new item".to_string(),
                });
            }
        }

        let mut new_item = item;
        new_item.width = new_width;
        self.items.insert(index, new_item);
        self.validate()?;
        Ok(())
    }

    /// Remove an item at the given index.
    pub fn remove(&mut self, index: usize) {
        let removed = self.items.remove(index);
        // Reclaim the removed item's width
        let _ = self.apply_deltas(-removed.width, None);
    }

    /// Move an item from one index to another.
    pub fn move_item(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        let item = self.items.remove(from);
        let to = if from < to { to - 1 } else { to };
        self.items.insert(to, item);
    }

    /// Set the container width, scaling existing items proportionally.
    pub fn set_container_width(&mut self, width: f64) -> Result<(), FlexError> {
        if width < 1.0 {
            return Err(FlexError::Unsatisfiable {
                reason: format!("new width ({width}) < 1"),
            });
        }
        let current_min: f64 = self.items.iter().map(|i| i.min_width).sum();
        if width < current_min {
            return Err(FlexError::ContainerTooSmall {
                width,
                min: current_min,
            });
        }

        let scale = width / self.container_width;
        for item in &mut self.items {
            item.width = item.clamp(item.width * scale);
        }
        self.container_width = width;
        self.validate()?;
        Ok(())
    }

    /// Resize a single item to a new width.
    pub fn resize(&mut self, index: usize, new_width: f64) -> Result<(), FlexError> {
        let item = self.items[index];
        let clamped = item.clamp(new_width);
        if clamped != new_width {
            return Err(FlexError::Unsatisfiable {
                reason: format!(
                    "width {new_width} out of bounds [{}, {}]",
                    item.min_width, item.max_width
                ),
            });
        }

        let delta = clamped - item.width;
        if delta > 0.0 {
            // Growing: need to reclaim space from other items
            let reclaim = self.unused_width() - delta;
            let leftover = self.apply_deltas(reclaim, None);
            // Now apply the growth to the target item
            let leftover = self.apply_deltas(leftover, Some(self.mask_all_except(index)));
            if leftover.abs() > 0.001 {
                return Err(FlexError::Unsatisfiable {
                    reason: "could not reclaim enough space".to_string(),
                });
            }
        } else {
            // Shrinking: redistribute space to other items
            let _ = self.apply_deltas(self.unused_width() - delta, None);
        }

        self.items[index].width = clamped;
        self.validate()?;
        Ok(())
    }

    /// Update constraints for an item.
    pub fn update_constraints(
        &mut self,
        index: usize,
        min: f64,
        max: f64,
    ) -> Result<(), FlexError> {
        let item = &mut self.items[index];
        item.min_width = min;
        item.max_width = max;
        let clamped = item.clamp(item.width);
        if clamped != item.width {
            self.resize(index, clamped)?;
        }
        self.validate()?;
        Ok(())
    }

    /// Get the container width.
    pub fn container_width(&self) -> f64 {
        self.container_width
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get an item by index.
    pub fn get(&self, index: usize) -> Option<&FlexItem> {
        self.items.get(index)
    }

    /// Get the total minimum width.
    pub fn min_width(&self) -> f64 {
        self.items.iter().map(|i| i.min_width).sum()
    }

    /// Get the total used width.
    pub fn used_width(&self) -> f64 {
        self.items.iter().map(|i| i.width).sum()
    }

    /// Get the unused width.
    pub fn unused_width(&self) -> f64 {
        self.container_width - self.used_width()
    }

    /// Allocate space for a new item, shrinking existing items.
    fn allocate(&mut self, min_width: f64, max_width: f64) -> f64 {
        let optimal = (self.container_width / (self.items.len() as f64 + 1.0))
            .min(self.container_width - self.min_width());
        // Match C# behavior: clamp(max(optimal, unused), min, max)
        optimal.max(self.unused_width()).clamp(min_width, max_width)
    }

    /// Apply deltas to items, returning leftover.
    ///
    /// Matches C# ApplyDeltasUnsafe behavior: returns the amount that
    /// could not be satisfied (0 if fully satisfied).
    fn apply_deltas(&mut self, total_delta: f64, mask: Option<Vec<bool>>) -> f64 {
        if total_delta.abs() < 0.001 {
            return 0.0;
        }

        let mask = mask.unwrap_or_else(|| vec![true; self.items.len()]);

        // Calculate available space for each item (normalized)
        let mut constraints: Vec<(f64, f64)> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if !mask[i] {
                    return (0.0, 0.0);
                }
                if total_delta > 0.0 {
                    // Growing: value = width, boundary = max_width
                    (
                        item.width / self.container_width,
                        item.max_width.min(self.container_width) / self.container_width,
                    )
                } else {
                    // Shrinking: value = width, boundary = min_width
                    (
                        item.width / self.container_width,
                        item.min_width / self.container_width,
                    )
                }
            })
            .collect();

        // Calculate available space
        let available_space: f64 = constraints
            .iter()
            .enumerate()
            .filter(|(i, _)| mask[*i])
            .map(|(_, (value, boundary))| boundary - value)
            .sum();

        if available_space.abs() < 0.001 {
            return total_delta.abs();
        }

        // Apply proportionally
        let normalized_delta = total_delta / self.container_width;
        let delta_factor = (normalized_delta / available_space).clamp(-1.0, 1.0);

        for (i, (value, boundary)) in constraints.iter_mut().enumerate() {
            if !mask[i] {
                continue;
            }
            let max_delta = *boundary - *value;
            *value += delta_factor * max_delta;
        }

        // Calculate leftover
        let leftover = (available_space - normalized_delta).max(0.0);

        // Denormalize and update items
        for (i, item) in self.items.iter_mut().enumerate() {
            if !mask[i] {
                continue;
            }
            let (value, boundary) = constraints[i];
            let denorm_value = value * self.container_width;
            let denorm_boundary = boundary * self.container_width;
            item.width = denorm_value;
            if total_delta <= 0.0 {
                item.min_width = denorm_boundary;
            }
            if total_delta >= 0.0 {
                item.max_width = denorm_boundary;
            }
        }

        leftover * self.container_width
    }

    fn mask_all_except(&self, index: usize) -> Vec<bool> {
        let mut mask = vec![false; self.items.len()];
        mask[index] = true;
        mask
    }

    fn validate(&self) -> Result<(), FlexError> {
        for (i, item) in self.items.iter().enumerate() {
            if item.width < item.min_width - 0.001 {
                return Err(FlexError::Unsatisfiable {
                    reason: format!(
                        "item {i} width {} < min_width {}",
                        item.width, item.min_width
                    ),
                });
            }
            if item.width > item.max_width + 0.001 {
                return Err(FlexError::Unsatisfiable {
                    reason: format!(
                        "item {i} width {} > max_width {}",
                        item.width, item.max_width
                    ),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_new() {
        let flex = Flex::new(100.0).unwrap();
        assert_eq!(flex.container_width(), 100.0);
        assert_eq!(flex.len(), 0);
    }

    #[test]
    fn flex_insert_single() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 10.0, 50.0).unwrap();
        assert_eq!(flex.len(), 1);
        assert!(flex.get(0).unwrap().width >= 10.0);
        assert!(flex.get(0).unwrap().width <= 50.0);
    }

    #[test]
    fn flex_insert_multiple() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 10.0, 50.0).unwrap();
        flex.insert(1, 10.0, 50.0).unwrap();
        flex.insert(2, 10.0, 50.0).unwrap();
        assert_eq!(flex.len(), 3);
        assert!(flex.used_width() <= 100.0);
    }

    #[test]
    fn flex_insert_overconstrained() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 40.0, 50.0).unwrap();
        flex.insert(1, 40.0, 50.0).unwrap();
        // 3rd item would exceed container (40+40+40=120 > 100)
        assert!(flex.insert(2, 40.0, 50.0).is_err());
    }

    #[test]
    fn flex_resize() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 10.0, 100.0).unwrap();
        flex.insert(1, 10.0, 100.0).unwrap();
        let _initial = flex.get(0).unwrap().width;
        flex.resize(0, 60.0).unwrap();
        assert!(flex.get(0).unwrap().width >= 59.0 && flex.get(0).unwrap().width <= 61.0);
    }

    #[test]
    fn flex_resize_violates_constraints() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 10.0, 30.0).unwrap();
        assert!(flex.resize(0, 50.0).is_err());
    }

    #[test]
    fn flex_remove() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 10.0, 100.0).unwrap();
        flex.insert(1, 10.0, 100.0).unwrap();
        flex.remove(0);
        assert_eq!(flex.len(), 1);
        assert!(flex.used_width() <= 100.0);
    }

    #[test]
    fn flex_set_container_width() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 10.0, 100.0).unwrap();
        flex.insert(1, 10.0, 100.0).unwrap();
        flex.set_container_width(200.0).unwrap();
        assert_eq!(flex.container_width(), 200.0);
        assert!(flex.used_width() <= 200.0);
    }

    #[test]
    fn flex_container_too_small() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 40.0, 50.0).unwrap();
        flex.insert(1, 40.0, 50.0).unwrap();
        assert!(flex.set_container_width(50.0).is_err());
    }

    #[test]
    fn flex_move_item() {
        let mut flex = Flex::new(100.0).unwrap();
        flex.insert(0, 10.0, 100.0).unwrap();
        flex.insert(1, 10.0, 100.0).unwrap();
        flex.insert(2, 10.0, 100.0).unwrap();
        let w0 = flex.get(0).unwrap().width;
        let w1 = flex.get(1).unwrap().width;
        let w2 = flex.get(2).unwrap().width;
        flex.move_item(0, 2);
        // Widths should be preserved after move
        assert!((flex.get(0).unwrap().width - w1).abs() < 0.001);
        assert!((flex.get(1).unwrap().width - w2).abs() < 0.001);
        assert!((flex.get(2).unwrap().width - w0).abs() < 0.001);
        assert!(flex.used_width() <= 100.0);
    }
}
