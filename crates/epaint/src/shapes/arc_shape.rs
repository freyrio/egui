use emath::TSTransform;

use crate::*;

/// Arc segment for path rendering
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ArcShape {
    /// The center point of the arc
    pub center: Pos2,
    /// The starting point of the arc
    pub start: Pos2,
    /// The ending point of the arc
    pub end: Pos2,
    /// The radii of the arc (x and y)
    pub radii: Vec2,
    /// The rotation of the arc in radians
    pub x_rotation: f32,
    /// Whether this is a large arc
    pub large_arc: bool,
    /// Whether this is a sweep arc
    pub sweep: bool,
    /// The starting angle in radians
    pub start_angle: f32,
    /// The fill color
    pub fill: Color32,
    /// The stroke
    pub stroke: PathStroke,
}

impl ArcShape {
    /// Create a new arc
    pub fn new(
        center: Pos2,
        start: Pos2,
        end: Pos2,
        radii: Vec2,
        x_rotation: f32,
        large_arc: bool,
        sweep: bool,
        fill: Color32,
        stroke: impl Into<PathStroke>,
    ) -> Self {
        let start_angle = (start.y - center.y).atan2(start.x - center.x);
        Self {
            center,
            start,
            end,
            radii,
            x_rotation,
            large_arc,
            sweep,
            start_angle,
            fill,
            stroke: stroke.into(),
        }
    }

    /// The visual bounding rectangle (includes stroke width)
    pub fn visual_bounding_rect(&self) -> Rect {
        let mut rect = Rect::from_points(&[self.start, self.end]);

        // Expand by the stroke width
        let stroke_expansion = self.stroke.width / 2.0;
        rect = rect.expand(stroke_expansion);

        // Include the center point
        rect = rect.union(Rect::from_pos(self.center));

        // Expand by the radii to account for the arc's curve
        rect = rect.expand2(self.radii);

        rect
    }

    /// Flatten the arc into line segments for rendering
    ///
    /// The `tolerance` parameter controls the maximum distance between the arc and its approximation.
    /// If `None`, a default tolerance will be used.
    pub fn flatten(&self, tolerance: Option<f32>) -> Vec<Pos2> {
        let tolerance = tolerance.unwrap_or(0.1);
        let mut points = Vec::new();

        // Calculate the number of segments based on the arc length and tolerance
        let arc_length = self.arc_length();
        let num_segments = (arc_length / tolerance).ceil() as usize;

        // Generate points along the arc
        for i in 0..=num_segments {
            let t = i as f32 / num_segments as f32;
            let point = self.point_at(t);
            points.push(point);
        }

        points
    }

    /// Calculate the length of the arc
    fn arc_length(&self) -> f32 {
        // Calculate arc length using radius and angle
        let radius = (self.radii.x + self.radii.y) / 2.0;
        let angle = self.angle();

        // Use the formula: L = r * θ where θ is in radians
        radius * angle
    }

    /// Calculate the angle of the arc in radians
    fn angle(&self) -> f32 {
        let start_angle = (self.start.y - self.center.y).atan2(self.start.x - self.center.x);
        let end_angle = (self.end.y - self.center.y).atan2(self.end.x - self.center.x);
        let mut angle = end_angle - start_angle;

        // Normalize angle to be positive
        if angle < 0.0 {
            angle += 2.0 * std::f32::consts::PI;
        }

        angle
    }

    /// Get a point on the arc at parameter t (0.0 to 1.0)
    fn point_at(&self, t: f32) -> Pos2 {
        let angle = self.angle();
        let current_angle = self.start_angle + angle * t;

        let x = self.center.x + self.radii.x * current_angle.cos();
        let y = self.center.y + self.radii.y * current_angle.sin();

        pos2(x, y)
    }

    /// Transform the arc with the given transform
    pub fn transform(&mut self, transform: TSTransform) {
        self.start = transform * self.start;
        self.end = transform * self.end;
        self.radii *= transform.scaling;
        self.stroke.width *= transform.scaling;
    }
}

impl Default for ArcShape {
    fn default() -> Self {
        Self {
            center: Pos2::ZERO,
            start: Pos2::ZERO,
            end: Pos2::ZERO,
            radii: Vec2::ZERO,
            x_rotation: 0.0,
            large_arc: false,
            sweep: false,
            start_angle: 0.0,
            fill: Color32::TRANSPARENT,
            stroke: PathStroke::default(),
        }
    }
}

impl From<ArcShape> for Shape {
    #[inline(always)]
    fn from(shape: ArcShape) -> Self {
        Self::Arc(shape)
    }
}

impl Eq for ArcShape {}

#[allow(clippy::derived_hash_with_manual_eq)]
impl std::hash::Hash for ArcShape {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self {
            center,
            start,
            end,
            radii,
            x_rotation,
            large_arc,
            sweep,
            start_angle,
            fill,
            stroke,
        } = self;
        emath::OrderedFloat(center.x).hash(state);
        emath::OrderedFloat(center.y).hash(state);
        emath::OrderedFloat(start.x).hash(state);
        emath::OrderedFloat(start.y).hash(state);
        emath::OrderedFloat(end.x).hash(state);
        emath::OrderedFloat(end.y).hash(state);
        emath::OrderedFloat(radii.x).hash(state);
        emath::OrderedFloat(radii.y).hash(state);
        emath::OrderedFloat(*x_rotation).hash(state);
        large_arc.hash(state);
        sweep.hash(state);
        emath::OrderedFloat(*start_angle).hash(state);
        fill.hash(state);
        stroke.hash(state);
    }
}

impl std::fmt::Display for ArcShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Arc(center: {}, start: {}, end: {}, radii: {}, x_rotation: {}, large_arc: {}, sweep: {})",
            self.center,
            self.start,
            self.end,
            self.radii,
            self.x_rotation,
            self.large_arc,
            self.sweep
        )
    }
}

#[test]
fn arc_shape_impl_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ArcShape>();
}
