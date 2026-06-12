//! Geometry primitives: rectangles, points, and sizes.

/// A 2D point with integer coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A 2D size with integer width and height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

/// A rectangle defined by position and size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn from_point_size(point: Point, size: Size) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn top_left(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn bottom_right(&self) -> Point {
        Point::new(self.x + self.width, self.y + self.height)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn clamp_within(&self, bounds: Rectangle) -> Rectangle {
        if self.is_empty() || bounds.is_empty() {
            return Rectangle::default();
        }

        let left = self.x.max(bounds.x);
        let top = self.y.max(bounds.y);
        let right = (self.x + self.width).min(bounds.x + bounds.width);
        let bottom = (self.y + self.height).min(bounds.y + bounds.height);

        Rectangle::new(left, top, (right - left).max(0), (bottom - top).max(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_contains_point() {
        let rect = Rectangle::new(0, 0, 100, 100);
        assert!(rect.contains(Point::new(50, 50)));
        assert!(rect.contains(Point::new(0, 0)));
        assert!(rect.contains(Point::new(100, 100)));
        assert!(!rect.contains(Point::new(101, 50)));
    }

    #[test]
    fn rectangle_from_point_size() {
        let point = Point::new(10, 20);
        let size = Size::new(30, 40);
        let rect = Rectangle::from_point_size(point, size);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 30);
        assert_eq!(rect.height, 40);
    }

    #[test]
    fn size_is_empty() {
        assert!(Size::new(0, 10).is_empty());
        assert!(Size::new(10, 0).is_empty());
        assert!(Size::new(-1, 10).is_empty());
        assert!(!Size::new(10, 10).is_empty());
    }

    #[test]
    fn rectangle_clamp_within_bounds() {
        let rect = Rectangle::new(-50, 10, 200, 150);
        let clamped = rect.clamp_within(Rectangle::new(0, 0, 100, 100));
        assert_eq!(clamped, Rectangle::new(0, 10, 100, 90));
    }
}
