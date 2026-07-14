//! Brush selection behavior for data filtering
//!
//! Provides rectangular selection for filtering data in visualizations.

use serde::{Deserialize, Serialize};

/// Type of brush selection
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushType {
    /// Horizontal brush (X-axis only)
    X,
    /// Vertical brush (Y-axis only)
    Y,
    /// Two-dimensional brush (both axes)
    #[default]
    XY,
}

/// A rectangular selection area
///
/// Represents the currently selected region in data or pixel coordinates.
///
/// # Example
///
/// ```
/// use makepad_d3::interaction::BrushSelection;
///
/// let selection = BrushSelection::new(100.0, 50.0, 300.0, 200.0);
/// assert!(selection.contains(200.0, 100.0));
/// assert!(!selection.contains(50.0, 100.0));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BrushSelection {
    /// Start X coordinate
    pub x0: f64,
    /// Start Y coordinate
    pub y0: f64,
    /// End X coordinate
    pub x1: f64,
    /// End Y coordinate
    pub y1: f64,
}

impl BrushSelection {
    /// Create a new brush selection
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self { x0, y0, x1, y1 }
    }

    /// Create a selection from a single point (zero-size)
    pub fn from_point(x: f64, y: f64) -> Self {
        Self {
            x0: x,
            y0: y,
            x1: x,
            y1: y,
        }
    }

    /// Create a selection from center and size
    pub fn from_center(cx: f64, cy: f64, width: f64, height: f64) -> Self {
        Self {
            x0: cx - width / 2.0,
            y0: cy - height / 2.0,
            x1: cx + width / 2.0,
            y1: cy + height / 2.0,
        }
    }

    /// Get normalized selection (x0 <= x1, y0 <= y1)
    pub fn normalized(&self) -> Self {
        Self {
            x0: self.x0.min(self.x1),
            y0: self.y0.min(self.y1),
            x1: self.x0.max(self.x1),
            y1: self.y0.max(self.y1),
        }
    }

    /// Width of the selection
    pub fn width(&self) -> f64 {
        (self.x1 - self.x0).abs()
    }

    /// Height of the selection
    pub fn height(&self) -> f64 {
        (self.y1 - self.y0).abs()
    }

    /// Area of the selection
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    /// Center X coordinate
    pub fn center_x(&self) -> f64 {
        (self.x0 + self.x1) / 2.0
    }

    /// Center Y coordinate
    pub fn center_y(&self) -> f64 {
        (self.y0 + self.y1) / 2.0
    }

    /// Center point
    pub fn center(&self) -> (f64, f64) {
        (self.center_x(), self.center_y())
    }

    /// Check if a point is inside the selection
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let norm = self.normalized();
        x >= norm.x0 && x <= norm.x1 && y >= norm.y0 && y <= norm.y1
    }

    /// Check if a point is inside the X range
    pub fn contains_x(&self, x: f64) -> bool {
        let (x0, x1) = (self.x0.min(self.x1), self.x0.max(self.x1));
        x >= x0 && x <= x1
    }

    /// Check if a point is inside the Y range
    pub fn contains_y(&self, y: f64) -> bool {
        let (y0, y1) = (self.y0.min(self.y1), self.y0.max(self.y1));
        y >= y0 && y <= y1
    }

    /// Check if selection is empty (zero area)
    pub fn is_empty(&self) -> bool {
        self.width() < 1e-10 || self.height() < 1e-10
    }

    /// Get X range (min, max)
    pub fn x_range(&self) -> (f64, f64) {
        (self.x0.min(self.x1), self.x0.max(self.x1))
    }

    /// Get Y range (min, max)
    pub fn y_range(&self) -> (f64, f64) {
        (self.y0.min(self.y1), self.y0.max(self.y1))
    }

    /// Intersect with another selection
    pub fn intersect(&self, other: &BrushSelection) -> Option<BrushSelection> {
        let a = self.normalized();
        let b = other.normalized();

        let x0 = a.x0.max(b.x0);
        let y0 = a.y0.max(b.y0);
        let x1 = a.x1.min(b.x1);
        let y1 = a.y1.min(b.y1);

        if x0 <= x1 && y0 <= y1 {
            Some(BrushSelection::new(x0, y0, x1, y1))
        } else {
            None
        }
    }

    /// Union with another selection
    pub fn union(&self, other: &BrushSelection) -> BrushSelection {
        let a = self.normalized();
        let b = other.normalized();

        BrushSelection::new(
            a.x0.min(b.x0),
            a.y0.min(b.y0),
            a.x1.max(b.x1),
            a.y1.max(b.y1),
        )
    }
}

/// Brush behavior state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BrushState {
    #[default]
    Idle,
    Selecting,
    Moving,
    ResizingN,
    ResizingS,
    ResizingE,
    ResizingW,
    ResizingNE,
    ResizingNW,
    ResizingSE,
    ResizingSW,
}

/// Brush behavior for rectangular selection
///
/// # Example
///
/// ```
/// use makepad_d3::interaction::{BrushBehavior, BrushType};
///
/// let mut brush = BrushBehavior::xy();
///
/// // Start selection at (100, 50)
/// brush.handle_start(100.0, 50.0);
///
/// // Drag to (200, 150)
/// brush.handle_move(200.0, 150.0);
///
/// // End selection
/// brush.handle_end();
///
/// // Get the selection
/// if let Some(sel) = brush.selection() {
///     println!("Selected region: ({}, {}) to ({}, {})", sel.x0, sel.y0, sel.x1, sel.y1);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct BrushBehavior {
    /// Type of brush
    brush_type: BrushType,
    /// Current selection
    selection: Option<BrushSelection>,
    /// Brush extent (bounds)
    extent: Option<BrushSelection>,
    /// Current state
    state: BrushState,
    /// Start position of current interaction
    start_pos: (f64, f64),
    /// Original selection when starting move/resize
    original_selection: Option<BrushSelection>,
    /// Handle size for resize detection
    handle_size: f64,
    /// Minimum selection size
    min_size: f64,
    /// Whether keyboard modifiers affect behavior
    keyboard_modifiers: bool,
}

impl Default for BrushBehavior {
    fn default() -> Self {
        Self::new(BrushType::XY)
    }
}

impl BrushBehavior {
    /// Create a new brush behavior
    pub fn new(brush_type: BrushType) -> Self {
        Self {
            brush_type,
            selection: None,
            extent: None,
            state: BrushState::Idle,
            start_pos: (0.0, 0.0),
            original_selection: None,
            handle_size: 8.0,
            min_size: 1.0,
            keyboard_modifiers: true,
        }
    }

    /// Create an X-axis only brush
    ///
    /// Note: For X-axis brushes, you should set an extent using `with_extent()`
    /// so the brush knows the full height of the selection area.
    pub fn x() -> Self {
        Self::new(BrushType::X)
    }

    /// Create a Y-axis only brush
    ///
    /// Note: For Y-axis brushes, you should set an extent using `with_extent()`
    /// so the brush knows the full width of the selection area.
    pub fn y() -> Self {
        Self::new(BrushType::Y)
    }

    /// Create a two-dimensional brush
    pub fn xy() -> Self {
        Self::new(BrushType::XY)
    }

    /// Set the brush extent (bounds)
    ///
    /// The extent defines the area where the brush can operate.
    /// For X and Y brush types, the extent also defines the span
    /// of the non-brushed dimension.
    ///
    /// **Important**: Setting an extent is required for `BrushType::X` and
    /// `BrushType::Y` to function correctly.
    pub fn with_extent(mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        self.extent = Some(BrushSelection::new(x0, y0, x1, y1));
        self
    }

    /// Check if the brush is properly configured
    ///
    /// Returns `false` if the brush type requires an extent but none is set.
    pub fn is_valid(&self) -> bool {
        match self.brush_type {
            BrushType::X | BrushType::Y => self.extent.is_some(),
            BrushType::XY => true,
        }
    }

    /// Get the extent if set
    pub fn extent(&self) -> Option<&BrushSelection> {
        self.extent.as_ref()
    }

    /// Set the handle size for resize detection
    pub fn with_handle_size(mut self, size: f64) -> Self {
        self.handle_size = size.max(1.0);
        self
    }

    /// Set the minimum selection size
    pub fn with_min_size(mut self, size: f64) -> Self {
        self.min_size = size.max(0.0);
        self
    }

    /// Get the current selection
    pub fn selection(&self) -> Option<BrushSelection> {
        self.selection
    }

    /// Get the brush type
    pub fn brush_type(&self) -> BrushType {
        self.brush_type
    }

    /// Check if currently selecting
    pub fn is_selecting(&self) -> bool {
        matches!(self.state, BrushState::Selecting)
    }

    /// Check if currently moving
    pub fn is_moving(&self) -> bool {
        matches!(self.state, BrushState::Moving)
    }

    /// Check if currently resizing
    pub fn is_resizing(&self) -> bool {
        matches!(
            self.state,
            BrushState::ResizingN
                | BrushState::ResizingS
                | BrushState::ResizingE
                | BrushState::ResizingW
                | BrushState::ResizingNE
                | BrushState::ResizingNW
                | BrushState::ResizingSE
                | BrushState::ResizingSW
        )
    }

    /// Check if interaction is active
    pub fn is_active(&self) -> bool {
        !matches!(self.state, BrushState::Idle)
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.selection = None;
        self.state = BrushState::Idle;
    }

    /// Set the selection programmatically
    pub fn set_selection(&mut self, selection: Option<BrushSelection>) {
        self.selection = selection.map(|s| self.constrain(s));
    }

    /// Handle the start of an interaction (mouse down)
    ///
    /// Returns `true` if the interaction was successfully started.
    /// Returns `false` if the brush is not properly configured (e.g., X/Y brush without extent).
    pub fn handle_start(&mut self, x: f64, y: f64) -> bool {
        // Validate configuration for X/Y brush types
        if !self.is_valid() {
            // X and Y brushes require an extent to be set
            return false;
        }

        self.start_pos = (x, y);

        // Check if clicking on existing selection handles
        if let Some(sel) = &self.selection {
            let resize_state = self.detect_resize_handle(x, y, sel);
            if resize_state != BrushState::Idle {
                self.state = resize_state;
                self.original_selection = self.selection;
                return true;
            }

            // Check if clicking inside selection (for move)
            if sel.contains(x, y) {
                self.state = BrushState::Moving;
                self.original_selection = self.selection;
                return true;
            }
        }

        // Start new selection
        self.state = BrushState::Selecting;
        self.selection = Some(BrushSelection::from_point(x, y));
        self.original_selection = None;
        true
    }

    /// Handle movement during interaction (mouse move)
    pub fn handle_move(&mut self, x: f64, y: f64) -> bool {
        match self.state {
            BrushState::Idle => false,
            BrushState::Selecting => {
                if let Some(mut sel) = self.selection {
                    match self.brush_type {
                        BrushType::X => {
                            sel.x1 = x;
                            // Use full extent height for X brush
                            if let Some(ext) = &self.extent {
                                sel.y0 = ext.y0;
                                sel.y1 = ext.y1;
                            }
                        }
                        BrushType::Y => {
                            sel.y1 = y;
                            // Use full extent width for Y brush
                            if let Some(ext) = &self.extent {
                                sel.x0 = ext.x0;
                                sel.x1 = ext.x1;
                            }
                        }
                        BrushType::XY => {
                            sel.x1 = x;
                            sel.y1 = y;
                        }
                    }
                    self.selection = Some(self.constrain(sel));
                }
                true
            }
            BrushState::Moving => {
                if let Some(orig) = &self.original_selection {
                    let dx = x - self.start_pos.0;
                    let dy = y - self.start_pos.1;
                    let moved =
                        BrushSelection::new(orig.x0 + dx, orig.y0 + dy, orig.x1 + dx, orig.y1 + dy);
                    self.selection = Some(self.constrain(moved));
                }
                true
            }
            _ => {
                // Handle resize states
                self.handle_resize(x, y)
            }
        }
    }

    /// Handle resize during interaction
    fn handle_resize(&mut self, x: f64, y: f64) -> bool {
        if let Some(orig) = &self.original_selection {
            let mut sel = *orig;
            let dx = x - self.start_pos.0;
            let dy = y - self.start_pos.1;

            match self.state {
                BrushState::ResizingN => sel.y0 = orig.y0 + dy,
                BrushState::ResizingS => sel.y1 = orig.y1 + dy,
                BrushState::ResizingE => sel.x1 = orig.x1 + dx,
                BrushState::ResizingW => sel.x0 = orig.x0 + dx,
                BrushState::ResizingNE => {
                    sel.y0 = orig.y0 + dy;
                    sel.x1 = orig.x1 + dx;
                }
                BrushState::ResizingNW => {
                    sel.y0 = orig.y0 + dy;
                    sel.x0 = orig.x0 + dx;
                }
                BrushState::ResizingSE => {
                    sel.y1 = orig.y1 + dy;
                    sel.x1 = orig.x1 + dx;
                }
                BrushState::ResizingSW => {
                    sel.y1 = orig.y1 + dy;
                    sel.x0 = orig.x0 + dx;
                }
                _ => return false,
            }

            self.selection = Some(self.constrain(sel));
            true
        } else {
            false
        }
    }

    /// Handle the end of an interaction (mouse up)
    pub fn handle_end(&mut self) -> Option<BrushSelection> {
        self.state = BrushState::Idle;
        self.original_selection = None;

        // Check if selection is too small
        if let Some(sel) = &self.selection {
            if sel.width() < self.min_size && sel.height() < self.min_size {
                self.selection = None;
            }
        }

        self.selection
    }

    /// Detect which resize handle is under the cursor
    fn detect_resize_handle(&self, x: f64, y: f64, sel: &BrushSelection) -> BrushState {
        let norm = sel.normalized();
        let h = self.handle_size;

        // Check corners first
        if (x - norm.x0).abs() <= h && (y - norm.y0).abs() <= h {
            return BrushState::ResizingNW;
        }
        if (x - norm.x1).abs() <= h && (y - norm.y0).abs() <= h {
            return BrushState::ResizingNE;
        }
        if (x - norm.x0).abs() <= h && (y - norm.y1).abs() <= h {
            return BrushState::ResizingSW;
        }
        if (x - norm.x1).abs() <= h && (y - norm.y1).abs() <= h {
            return BrushState::ResizingSE;
        }

        // Check edges
        if (y - norm.y0).abs() <= h && x >= norm.x0 && x <= norm.x1 {
            return BrushState::ResizingN;
        }
        if (y - norm.y1).abs() <= h && x >= norm.x0 && x <= norm.x1 {
            return BrushState::ResizingS;
        }
        if (x - norm.x0).abs() <= h && y >= norm.y0 && y <= norm.y1 {
            return BrushState::ResizingW;
        }
        if (x - norm.x1).abs() <= h && y >= norm.y0 && y <= norm.y1 {
            return BrushState::ResizingE;
        }

        BrushState::Idle
    }

    /// Constrain selection to extent
    fn constrain(&self, mut sel: BrushSelection) -> BrushSelection {
        if let Some(ext) = &self.extent {
            let e = ext.normalized();
            sel.x0 = sel.x0.clamp(e.x0, e.x1);
            sel.x1 = sel.x1.clamp(e.x0, e.x1);
            sel.y0 = sel.y0.clamp(e.y0, e.y1);
            sel.y1 = sel.y1.clamp(e.y0, e.y1);
        }
        sel
    }

    /// Get cursor style for current position
    pub fn cursor_at(&self, x: f64, y: f64) -> BrushCursor {
        if let Some(sel) = &self.selection {
            let state = self.detect_resize_handle(x, y, sel);
            match state {
                BrushState::ResizingN | BrushState::ResizingS => BrushCursor::NSResize,
                BrushState::ResizingE | BrushState::ResizingW => BrushCursor::EWResize,
                BrushState::ResizingNE | BrushState::ResizingSW => BrushCursor::NESWResize,
                BrushState::ResizingNW | BrushState::ResizingSE => BrushCursor::NWSEResize,
                _ if sel.contains(x, y) => BrushCursor::Move,
                _ => BrushCursor::Crosshair,
            }
        } else {
            BrushCursor::Crosshair
        }
    }
}

/// Cursor style hint for brush interaction
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BrushCursor {
    /// Default cursor for creating selection
    #[default]
    Crosshair,
    /// Cursor for moving selection
    Move,
    /// North-South resize cursor
    NSResize,
    /// East-West resize cursor
    EWResize,
    /// Northeast-Southwest resize cursor
    NESWResize,
    /// Northwest-Southeast resize cursor
    NWSEResize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brush_selection_new() {
        let sel = BrushSelection::new(10.0, 20.0, 100.0, 80.0);
        assert_eq!(sel.x0, 10.0);
        assert_eq!(sel.y0, 20.0);
        assert_eq!(sel.x1, 100.0);
        assert_eq!(sel.y1, 80.0);
    }

    #[test]
    fn test_brush_selection_dimensions() {
        let sel = BrushSelection::new(0.0, 0.0, 100.0, 50.0);
        assert_eq!(sel.width(), 100.0);
        assert_eq!(sel.height(), 50.0);
        assert_eq!(sel.area(), 5000.0);
        assert_eq!(sel.center(), (50.0, 25.0));
    }

    #[test]
    fn test_brush_selection_contains() {
        let sel = BrushSelection::new(0.0, 0.0, 100.0, 50.0);
        assert!(sel.contains(50.0, 25.0));
        assert!(sel.contains(0.0, 0.0));
        assert!(sel.contains(100.0, 50.0));
        assert!(!sel.contains(-1.0, 25.0));
        assert!(!sel.contains(101.0, 25.0));
    }

    #[test]
    fn test_brush_selection_normalized() {
        let sel = BrushSelection::new(100.0, 80.0, 10.0, 20.0);
        let norm = sel.normalized();
        assert_eq!(norm.x0, 10.0);
        assert_eq!(norm.y0, 20.0);
        assert_eq!(norm.x1, 100.0);
        assert_eq!(norm.y1, 80.0);
    }

    #[test]
    fn test_brush_selection_intersect() {
        let a = BrushSelection::new(0.0, 0.0, 100.0, 100.0);
        let b = BrushSelection::new(50.0, 50.0, 150.0, 150.0);
        let i = a.intersect(&b).unwrap();
        assert_eq!(i.x0, 50.0);
        assert_eq!(i.y0, 50.0);
        assert_eq!(i.x1, 100.0);
        assert_eq!(i.y1, 100.0);
    }

    #[test]
    fn test_brush_selection_no_intersect() {
        let a = BrushSelection::new(0.0, 0.0, 50.0, 50.0);
        let b = BrushSelection::new(100.0, 100.0, 150.0, 150.0);
        assert!(a.intersect(&b).is_none());
    }

    #[test]
    fn test_brush_selection_union() {
        let a = BrushSelection::new(0.0, 0.0, 50.0, 50.0);
        let b = BrushSelection::new(25.0, 25.0, 100.0, 100.0);
        let u = a.union(&b);
        assert_eq!(u.x0, 0.0);
        assert_eq!(u.y0, 0.0);
        assert_eq!(u.x1, 100.0);
        assert_eq!(u.y1, 100.0);
    }

    #[test]
    fn test_brush_behavior_xy() {
        let mut brush = BrushBehavior::xy();
        assert!(brush.selection().is_none());

        assert!(brush.handle_start(10.0, 10.0)); // Returns true for XY brush
        assert!(brush.is_selecting());

        brush.handle_move(100.0, 80.0);
        let sel = brush.selection().unwrap();
        assert_eq!(sel.x0, 10.0);
        assert_eq!(sel.y0, 10.0);
        assert_eq!(sel.x1, 100.0);
        assert_eq!(sel.y1, 80.0);

        let final_sel = brush.handle_end();
        assert!(final_sel.is_some());
        assert!(!brush.is_active());
    }

    #[test]
    fn test_brush_validation() {
        // XY brush is always valid
        let xy_brush = BrushBehavior::xy();
        assert!(xy_brush.is_valid());

        // X brush without extent is invalid
        let x_brush = BrushBehavior::x();
        assert!(!x_brush.is_valid());

        // X brush with extent is valid
        let x_brush_with_extent = BrushBehavior::x().with_extent(0.0, 0.0, 100.0, 100.0);
        assert!(x_brush_with_extent.is_valid());

        // Y brush without extent is invalid
        let y_brush = BrushBehavior::y();
        assert!(!y_brush.is_valid());

        // Y brush with extent is valid
        let y_brush_with_extent = BrushBehavior::y().with_extent(0.0, 0.0, 100.0, 100.0);
        assert!(y_brush_with_extent.is_valid());
    }

    #[test]
    fn test_brush_x_without_extent_fails() {
        let mut brush = BrushBehavior::x();
        // Should fail to start without extent
        assert!(!brush.handle_start(50.0, 50.0));
        assert!(!brush.is_active());
    }

    #[test]
    fn test_brush_y_without_extent_fails() {
        let mut brush = BrushBehavior::y();
        // Should fail to start without extent
        assert!(!brush.handle_start(50.0, 50.0));
        assert!(!brush.is_active());
    }

    #[test]
    fn test_brush_behavior_x() {
        let mut brush = BrushBehavior::x().with_extent(0.0, 0.0, 500.0, 300.0);

        brush.handle_start(50.0, 150.0);
        brush.handle_move(200.0, 100.0);

        let sel = brush.selection().unwrap();
        // Y should span full extent for X brush
        assert_eq!(sel.y0, 0.0);
        assert_eq!(sel.y1, 300.0);
        assert_eq!(sel.x0, 50.0);
        assert_eq!(sel.x1, 200.0);
    }

    #[test]
    fn test_brush_behavior_y() {
        let mut brush = BrushBehavior::y().with_extent(0.0, 0.0, 500.0, 300.0);

        brush.handle_start(250.0, 50.0);
        brush.handle_move(100.0, 200.0);

        let sel = brush.selection().unwrap();
        // X should span full extent for Y brush
        assert_eq!(sel.x0, 0.0);
        assert_eq!(sel.x1, 500.0);
        assert_eq!(sel.y0, 50.0);
        assert_eq!(sel.y1, 200.0);
    }

    #[test]
    fn test_brush_behavior_move() {
        let mut brush = BrushBehavior::xy();

        // Create initial selection
        assert!(brush.handle_start(0.0, 0.0));
        brush.handle_move(100.0, 100.0);
        brush.handle_end();

        // Start move inside selection
        assert!(brush.handle_start(50.0, 50.0));
        assert!(brush.is_moving());

        // Move selection
        brush.handle_move(60.0, 70.0);
        let sel = brush.selection().unwrap();
        // Selection should move by (10, 20)
        assert_eq!(sel.x0, 10.0);
        assert_eq!(sel.y0, 20.0);
        assert_eq!(sel.x1, 110.0);
        assert_eq!(sel.y1, 120.0);
    }

    #[test]
    fn test_brush_behavior_clear() {
        let mut brush = BrushBehavior::xy();
        assert!(brush.handle_start(0.0, 0.0));
        brush.handle_move(100.0, 100.0);
        brush.handle_end();

        assert!(brush.selection().is_some());
        brush.clear();
        assert!(brush.selection().is_none());
    }

    #[test]
    fn test_brush_behavior_extent_constraint() {
        let mut brush = BrushBehavior::xy().with_extent(0.0, 0.0, 100.0, 100.0);

        assert!(brush.handle_start(10.0, 10.0));
        brush.handle_move(200.0, 200.0); // Beyond extent

        let sel = brush.selection().unwrap();
        assert!(sel.x1 <= 100.0);
        assert!(sel.y1 <= 100.0);
    }

    #[test]
    fn test_brush_cursor() {
        let mut brush = BrushBehavior::xy();

        // No selection - crosshair
        assert_eq!(brush.cursor_at(50.0, 50.0), BrushCursor::Crosshair);

        // Create selection
        assert!(brush.handle_start(0.0, 0.0));
        brush.handle_move(100.0, 100.0);
        brush.handle_end();

        // Inside selection - move cursor
        assert_eq!(brush.cursor_at(50.0, 50.0), BrushCursor::Move);
    }

    #[test]
    fn test_brush_type() {
        assert_eq!(BrushBehavior::x().brush_type(), BrushType::X);
        assert_eq!(BrushBehavior::y().brush_type(), BrushType::Y);
        assert_eq!(BrushBehavior::xy().brush_type(), BrushType::XY);
    }
}
