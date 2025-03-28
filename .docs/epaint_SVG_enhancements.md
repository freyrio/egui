# Blueprint: Enhanced epaint for SVG Rendering Compatibility

## Overview

This document outlines a comprehensive plan to extend epaint's rendering capabilities to support all SVG 1.1 features while maintaining backward compatibility. The focus is exclusively on rendering capabilities, not parsing.

## 1. Path Enhancements

### 1.1 ArcShape Implementation

```rust
/// Arc segment for path rendering
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ArcShape {
    /// Start point of the arc
    pub start: Pos2,

    /// End point of the arc
    pub end: Pos2,

    /// Radii of the ellipse (x and y components)
    pub radii: Vec2,

    /// Rotation of the ellipse in radians
    pub x_rotation: f32,

    /// Use the large arc (true) or small arc (false)
    pub large_arc: bool,

    /// Sweep the arc clockwise (true) or counterclockwise (false)
    pub sweep: bool,

    /// Fill color for closed arcs
    pub fill: Color32,

    /// Stroke properties
    pub stroke: PathStroke,
}

impl ArcShape {
    /// Create a new arc
    pub fn new(
        start: Pos2,
        end: Pos2,
        radii: Vec2,
        x_rotation: f32,
        large_arc: bool,
        sweep: bool,
        fill: Color32,
        stroke: impl Into<PathStroke>,
    ) -> Self {
        Self {
            start,
            end,
            radii,
            x_rotation,
            large_arc,
            sweep,
            fill,
            stroke: stroke.into(),
        }
    }

    /// The visual bounding rectangle (includes stroke width)
    pub fn visual_bounding_rect(&self) -> Rect {
        // Implementation that calculates the bounding rectangle for an arc
        // This requires computing the extreme points of the arc
    }

    /// Flatten the arc into line segments for rendering
    pub fn flatten(&self, tolerance: Option<f32>) -> Vec<Pos2> {
        // Implementation that converts the arc to a series of points
        // based on the specified tolerance
    }
}

impl From<ArcShape> for Shape {
    #[inline(always)]
    fn from(shape: ArcShape) -> Self {
        Self::Arc(shape)
    }
}
```

### 1.2 Enhanced Bézier Support

#### 1.2.1 Smooth Cubic Bézier

```rust
impl CubicBezierShape {
    /// Create a smooth cubic Bézier curve that continues from the previous curve
    /// The first control point is reflected from the previous curve's last control point
    pub fn smooth_from(
        previous: &CubicBezierShape,
        control2: Pos2,
        end: Pos2,
        closed: bool,
        fill: Color32,
        stroke: impl Into<PathStroke>,
    ) -> Self {
        // Calculate the reflection of the previous curve's last control point
        let prev_control = previous.points[2];
        let prev_end = previous.points[3];
        let control1 = prev_end + (prev_end - prev_control);

        Self {
            points: [prev_end, control1, control2, end],
            closed,
            fill,
            stroke: stroke.into(),
        }
    }
}
```

#### 1.2.2 Smooth Quadratic Bézier

```rust
impl QuadraticBezierShape {
    /// Create a smooth quadratic Bézier curve that continues from the previous curve
    /// The control point is reflected from the previous curve's control point
    pub fn smooth_from(
        previous: &QuadraticBezierShape,
        end: Pos2,
        closed: bool,
        fill: Color32,
        stroke: impl Into<PathStroke>,
    ) -> Self {
        // Calculate the reflection of the previous curve's control point
        let prev_control = previous.points[1];
        let prev_end = previous.points[2];
        let control = prev_end + (prev_end - prev_control);

        Self {
            points: [prev_end, control, end],
            closed,
            fill,
            stroke: stroke.into(),
        }
    }
}
```

### 1.3 Enhanced PathShape

```rust
/// Enhanced PathShape to support SVG-compatible path operations
pub struct PathShape {
    // Existing fields
    pub points: Vec<Pos2>,
    pub closed: bool,
    pub fill: Color32,
    pub stroke: PathStroke,

    // New fields
    /// Fill rule for determining the inside of the path
    pub fill_rule: FillRule,

    /// Path segments that require special handling (arcs, curves, etc.)
    pub segments: Vec<PathSegment>,
}

/// The rule to determine the inside of a path (for fill)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillRule {
    /// Non-zero winding rule (default SVG behavior)
    NonZero,

    /// Even-odd rule
    EvenOdd,
}

/// Different types of path segments
#[derive(Clone, Debug, PartialEq)]
pub enum PathSegment {
    /// Regular line segment between points
    Line(usize, usize), // Indices into the points array

    /// Cubic Bézier curve
    CubicBezier {
        start_idx: usize,
        control1_idx: usize,
        control2_idx: usize,
        end_idx: usize,
    },

    /// Smooth cubic Bézier curve (S command)
    SmoothCubicBezier {
        start_idx: usize,    // Previous endpoint
        control2_idx: usize, // Second control point
        end_idx: usize,      // End point
    },

    /// Quadratic Bézier curve
    QuadraticBezier {
        start_idx: usize,
        control_idx: usize,
        end_idx: usize,
    },

    /// Smooth quadratic Bézier curve (T command)
    SmoothQuadraticBezier {
        start_idx: usize, // Previous endpoint
        end_idx: usize,   // End point
    },

    /// Arc segment
    Arc {
        start_idx: usize,
        end_idx: usize,
        radii: Vec2,
        x_rotation: f32,
        large_arc: bool,
        sweep: bool,
    },
}

impl PathShape {
    // Add methods to construct and manipulate the enhanced path

    /// Add a smooth cubic Bézier segment to the path
    pub fn add_smooth_cubic_bezier(&mut self, control2: Pos2, end: Pos2) -> &mut Self {
        let start_idx = self.points.len() - 1;
        self.points.push(control2);
        self.points.push(end);
        let control2_idx = start_idx + 1;
        let end_idx = start_idx + 2;

        self.segments.push(PathSegment::SmoothCubicBezier {
            start_idx,
            control2_idx,
            end_idx,
        });

        self
    }

    /// Add an arc segment to the path
    pub fn add_arc(
        &mut self,
        end: Pos2,
        radii: Vec2,
        x_rotation: f32,
        large_arc: bool,
        sweep: bool
    ) -> &mut Self {
        let start_idx = self.points.len() - 1;
        self.points.push(end);
        let end_idx = self.points.len() - 1;

        self.segments.push(PathSegment::Arc {
            start_idx,
            end_idx,
            radii,
            x_rotation,
            large_arc,
            sweep,
        });

        self
    }

    // Add other methods for different segment types
}
```

## 2. Stroke Enhancements

```rust
/// How the end of a line should be rendered
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum LineCap {
    /// Squared off at the end point (SVG "butt")
    Butt,

    /// Rounded end (SVG "round")
    Round,

    /// Squared off beyond the end point by half line width (SVG "square")
    Square,
}

/// How line segments join
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum LineJoin {
    /// Sharp corner (SVG "miter")
    Miter,

    /// Rounded corner (SVG "round")
    Round,

    /// Beveled corner (SVG "bevel")
    Bevel,
}

/// Enhanced Stroke structure with SVG-compatible properties
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Stroke {
    pub width: f32,
    pub color: Color32,

    // New properties
    pub cap: LineCap,
    pub join: LineJoin,
    pub miter_limit: f32,
}

impl Stroke {
    /// Same as [`Stroke::default`].
    pub const NONE: Self = Self {
        width: 0.0,
        color: Color32::TRANSPARENT,
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        miter_limit: 4.0, // SVG default
    };

    /// Create a new stroke with line cap and join properties
    pub fn new_with_props(
        width: impl Into<f32>,
        color: impl Into<Color32>,
        cap: LineCap,
        join: LineJoin,
        miter_limit: f32,
    ) -> Self {
        Self {
            width: width.into(),
            color: color.into(),
            cap,
            join,
            miter_limit,
        }
    }

    // Keep existing methods and add new ones

    /// Set the line cap
    pub fn with_line_cap(mut self, cap: LineCap) -> Self {
        self.cap = cap;
        self
    }

    /// Set the line join
    pub fn with_line_join(mut self, join: LineJoin) -> Self {
        self.join = join;
        self
    }

    /// Set the miter limit
    pub fn with_miter_limit(mut self, miter_limit: f32) -> Self {
        self.miter_limit = miter_limit;
        self
    }
}

// Similarly enhance PathStroke
```

## 3. Gradient Support

```rust
/// A color stop for gradients
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GradientStop {
    /// Offset position (0.0 to 1.0)
    pub offset: f32,

    /// Color at this offset
    pub color: Color32,
}

/// Types of gradients
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Gradient {
    /// Linear gradient defined by start and end points
    Linear {
        start: Pos2,
        end: Pos2,
        stops: Vec<GradientStop>,
    },

    /// Radial gradient defined by center, radius, and optional focus
    Radial {
        center: Pos2,
        radius: f32,
        focus: Option<Pos2>, // If None, same as center
        stops: Vec<GradientStop>,
    },
}

impl Gradient {
    /// Create a new linear gradient
    pub fn linear(start: Pos2, end: Pos2, stops: Vec<GradientStop>) -> Self {
        Self::Linear { start, end, stops }
    }

    /// Create a new radial gradient
    pub fn radial(center: Pos2, radius: f32, stops: Vec<GradientStop>) -> Self {
        Self::Radial {
            center,
            radius,
            focus: None,
            stops
        }
    }

    /// Create a new radial gradient with focus
    pub fn radial_with_focus(center: Pos2, radius: f32, focus: Pos2, stops: Vec<GradientStop>) -> Self {
        Self::Radial {
            center,
            radius,
            focus: Some(focus),
            stops
        }
    }

    /// Convert this gradient to a color callback for use with ColorMode::UV
    pub fn to_color_callback(&self) -> Arc<dyn Fn(Rect, Pos2) -> Color32 + Send + Sync> {
        match self {
            Self::Linear { start, end, stops } => {
                let stops = stops.clone();
                Arc::new(move |_rect, pos| {
                    // Compute distance along gradient line
                    let line_vec = *end - *start;
                    let point_vec = pos - *start;
                    let t = point_vec.dot(line_vec) / line_vec.length_sq();

                    // Find appropriate color
                    gradient_color_at(t, &stops)
                })
            }
            Self::Radial { center, radius, focus, stops } => {
                let stops = stops.clone();
                let focus = *focus;
                Arc::new(move |_rect, pos| {
                    let t = if let Some(focus_point) = focus {
                        // Complex computation for focused radial gradients
                        // (This is simplified - actual implementation would be more complex)
                        let fp_to_pos = pos - focus_point;
                        let fp_to_center = *center - focus_point;
                        let a = fp_to_pos.length_sq();
                        let b = 2.0 * fp_to_pos.dot(fp_to_center);
                        let c = fp_to_center.length_sq() - radius.powi(2);

                        // Solve quadratic equation to find intersection
                        // Simplified for blueprint
                        (pos - *center).length() / *radius
                    } else {
                        // Simple case: focus is at center
                        (pos - *center).length() / *radius
                    };

                    gradient_color_at(t, &stops)
                })
            }
        }
    }
}

/// Helper function to interpolate color at a position within a gradient
fn gradient_color_at(t: f32, stops: &[GradientStop]) -> Color32 {
    // Find the two stops that contain t
    if t <= 0.0 {
        return stops.first().map_or(Color32::TRANSPARENT, |s| s.color);
    }
    if t >= 1.0 {
        return stops.last().map_or(Color32::TRANSPARENT, |s| s.color);
    }

    // Find the stops that bracket t
    let mut i = 0;
    while i < stops.len() - 1 && stops[i + 1].offset < t {
        i += 1;
    }

    if i >= stops.len() - 1 {
        return stops.last().map_or(Color32::TRANSPARENT, |s| s.color);
    }

    let stop1 = &stops[i];
    let stop2 = &stops[i + 1];

    // Interpolate between the two stops
    let t_norm = (t - stop1.offset) / (stop2.offset - stop1.offset);
    interpolate_color(stop1.color, stop2.color, t_norm)
}

/// Helper function to linearly interpolate between two colors
fn interpolate_color(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let r = lerp(c1.r() as f32, c2.r() as f32, t) as u8;
    let g = lerp(c1.g() as f32, c2.g() as f32, t) as u8;
    let b = lerp(c1.b() as f32, c2.b() as f32, t) as u8;
    let a = lerp(c1.a() as f32, c2.a() as f32, t) as u8;
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// Enhance the ColorMode to support gradients directly
pub enum ColorMode {
    /// The entire path is one solid color, this is the default.
    Solid(Color32),

    /// Provide a callback which takes in the path's bounding box and a position and converts it to a color.
    UV(Arc<dyn Fn(Rect, Pos2) -> Color32 + Send + Sync>),

    /// A gradient (linear or radial)
    Gradient(Gradient),
}

impl ColorMode {
    /// Create a gradient color mode
    pub fn gradient(gradient: Gradient) -> Self {
        Self::UV(gradient.to_color_callback())
    }
}
```

## 4. Transform Enhancements

```rust
/// Enhanced transform to include all SVG transform capabilities
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// The transformation matrix (3x3 for 2D homogeneous coordinates)
    /// Stored in column-major order: [a, c, e, b, d, f, 0, 0, 1]
    /// Represents the matrix:
    /// | a c e |
    /// | b d f |
    /// | 0 0 1 |
    pub matrix: [f32; 9],
}

impl Transform {
    /// Identity transform
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    /// Create a translation transform
    pub fn translate(tx: f32, ty: f32) -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, tx, ty, 1.0],
        }
    }

    /// Create a scaling transform
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            matrix: [sx, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Create a rotation transform (in radians)
    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            matrix: [cos, sin, 0.0, -sin, cos, 0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Create a skew transform in the X direction (in radians)
    pub fn skew_x(angle: f32) -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, angle.tan(), 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Create a skew transform in the Y direction (in radians)
    pub fn skew_y(angle: f32) -> Self {
        Self {
            matrix: [1.0, angle.tan(), 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Compose two transforms (apply this transform after other)
    pub fn compose(&self, other: &Self) -> Self {
        // Matrix multiplication for 3x3 matrices
        let mut result = [0.0; 9];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i*3 + j] += self.matrix[i*3 + k] * other.matrix[k*3 + j];
                }
            }
        }
        Self { matrix: result }
    }

    /// Apply this transform to a point
    pub fn transform_point(&self, point: Pos2) -> Pos2 {
        let x = self.matrix[0] * point.x + self.matrix[3] * point.y + self.matrix[6];
        let y = self.matrix[1] * point.x + self.matrix[4] * point.y + self.matrix[7];
        Pos2::new(x, y)
    }
}

// Enhance Shape with an additional transform method
impl Shape {
    /// Apply a general transform to this shape
    pub fn with_transform(&mut self, transform: Transform) {
        match self {
            Self::Noop => {}
            Self::Vec(shapes) => {
                for shape in shapes {
                    shape.with_transform(transform);
                }
            }
            Self::Circle(circle_shape) => {
                circle_shape.center = transform.transform_point(circle_shape.center);
                // Note: This is simplified - a proper implementation would handle non-uniform scaling
                let scale_factor = ((transform.matrix[0].powi(2) + transform.matrix[1].powi(2)).sqrt() +
                                    (transform.matrix[3].powi(2) + transform.matrix[4].powi(2)).sqrt()) / 2.0;
                circle_shape.radius *= scale_factor;
            }
            // Handle other shape types similarly
            _ => {
                // Default fallback to the existing transform method for backward compatibility
                let ts_transform = TSTransform {
                    translation: Vec2::new(transform.matrix[6], transform.matrix[7]),
                    scaling: ((transform.matrix[0].powi(2) + transform.matrix[1].powi(2)).sqrt() +
                             (transform.matrix[3].powi(2) + transform.matrix[4].powi(2)).sqrt()) / 2.0,
                };
                self.transform(ts_transform);
            }
        }
    }
}
```

## 5. Text on Path Support

```rust
/// Enhanced TextShape to support text on a path
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TextShape {
    // Existing fields
    pub pos: Pos2,
    pub galley: Arc<Galley>,
    pub underline: Stroke,
    pub fallback_color: Color32,
    pub override_text_color: Option<Color32>,
    pub opacity_factor: f32,
    pub angle: f32,

    // New fields
    pub path: Option<TextPath>,
}

/// Define how text follows a path
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TextPath {
    /// The points defining the path
    pub points: Vec<Pos2>,

    /// Offset along the path
    pub offset: f32,

    /// Spacing adjustment
    pub spacing: f32,

    /// Whether to rotate characters to align with the path tangent
    pub rotate_chars: bool,
}

impl TextShape {
    /// Create a text shape that follows a path
    pub fn on_path(
        galley: Arc<Galley>,
        path_points: Vec<Pos2>,
        offset: f32,
        fallback_color: Color32,
    ) -> Self {
        Self {
            pos: Pos2::ZERO, // Not used when on a path
            galley,
            underline: Stroke::NONE,
            fallback_color,
            override_text_color: None,
            opacity_factor: 1.0,
            angle: 0.0,
            path: Some(TextPath {
                points: path_points,
                offset,
                spacing: 1.0,
                rotate_chars: true,
            }),
        }
    }
}
```

## 6. Blend Mode Support

```rust
/// Blend modes for compositing
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BlendMode {
    /// Normal (source over destination)
    Normal,

    /// Multiply source and destination
    Multiply,

    /// Screen blending
    Screen,

    /// Overlay blending
    Overlay,

    /// Darken (take the darker of source and destination)
    Darken,

    /// Lighten (take the lighter of source and destination)
    Lighten,

    /// Color dodge
    ColorDodge,

    /// Color burn
    ColorBurn,

    /// Hard light
    HardLight,

    /// Soft light
    SoftLight,

    /// Difference
    Difference,

    /// Exclusion
    Exclusion,

    /// Hue
    Hue,

    /// Saturation
    Saturation,

    /// Color
    Color,

    /// Luminosity
    Luminosity,
}

/// Group of shapes with a specific blend mode
#[derive(Clone, Debug, PartialEq)]
pub struct BlendGroup {
    /// The shapes in this group
    pub shapes: Vec<Shape>,

    /// The blend mode to apply
    pub blend_mode: BlendMode,

    /// Optional opacity for the entire group
    pub opacity: Option<f32>,
}

impl BlendGroup {
    /// Create a new blend group
    pub fn new(shapes: Vec<Shape>, blend_mode: BlendMode) -> Self {
        Self {
            shapes,
            blend_mode,
            opacity: None,
        }
    }

    /// Set the opacity for this group
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }
}

// Add BlendGroup to the Shape enum
impl From<BlendGroup> for Shape {
    #[inline(always)]
    fn from(group: BlendGroup) -> Self {
        Self::BlendGroup(group)
    }
}

// Update Shape enum
pub enum Shape {
    // Existing variants...

    /// A group of shapes with a specific blend mode
    BlendGroup(BlendGroup),
}
```

## 7. Masking Support

```rust
/// A mask that defines the visible area for child shapes
#[derive(Clone, Debug, PartialEq)]
pub struct Mask {
    /// The shape that defines the mask
    pub mask_shape: Box<Shape>,

    /// The shapes being masked
    pub shapes: Vec<Shape>,

    /// How the mask is applied
    pub mask_type: MaskType,

    /// Whether to clip to the mask shape's border
    pub clip_to_border: bool,
}

/// How the mask is applied
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MaskType {
    /// Use alpha channel of the mask (SVG "alpha")
    Alpha,

    /// Use luminance of the mask (SVG "luminance")
    Luminance,
}

impl Mask {
    /// Create a new alpha mask
    pub fn alpha(mask_shape: impl Into<Shape>, shapes: Vec<Shape>) -> Self {
        Self {
            mask_shape: Box::new(mask_shape.into()),
            shapes,
            mask_type: MaskType::Alpha,
            clip_to_border: false,
        }
    }

    /// Create a new luminance mask
    pub fn luminance(mask_shape: impl Into<Shape>, shapes: Vec<Shape>) -> Self {
        Self {
            mask_shape: Box::new(mask_shape.into()),
            shapes,
            mask_type: MaskType::Luminance,
            clip_to_border: false,
        }
    }

    /// Set whether to clip to the mask shape's border
    pub fn with_clip_to_border(mut self, clip: bool) -> Self {
        self.clip_to_border = clip;
        self
    }
}

// Add Mask to the Shape enum
impl From<Mask> for Shape {
    #[inline(always)]
    fn from(mask: Mask) -> Self {
        Self::Mask(mask)
    }
}

// Update Shape enum
pub enum Shape {
    // Existing variants...

    /// A masked group of shapes
    Mask(Mask),
}
```

## 8. Filter Effects

```rust
/// Filter effects that can be applied to shapes
#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    /// The shapes to apply the filter to
    pub shapes: Vec<Shape>,

    /// The filter operations to apply
    pub operations: Vec<FilterOperation>,
}

/// Types of filter operations
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum FilterOperation {
    /// Gaussian blur
    GaussianBlur {
        /// Standard deviation in X direction
        std_dev_x: f32,

        /// Standard deviation in Y direction
        std_dev_y: f32,
    },

    /// Color matrix filter
    ColorMatrix {
        /// 5x4 color matrix (20 values)
        matrix: [f32; 20],
    },

    /// Displacement map
    DisplacementMap {
        /// Scale factor
        scale: f32,

        /// Channel to use for X displacement
        x_channel: ColorChannel,

        /// Channel to use for Y displacement
        y_channel: ColorChannel,

        /// The displacement map
        map: Box<Shape>,
    },

    /// Flood (fill an area with a color)
    Flood {
        /// The color to fill with
        color: Color32,
    },

    /// Composite operation
    Composite {
        /// The background shapes
        background: Box<Shape>,

        /// The operation to use for compositing
        operator: CompositeOperator,
    },

    /// Morphology operations
    Morphology {
        /// The operator to use
        operator: MorphologyOperator,

        /// Radius
        radius: f32,
    },
}

/// Color channels for displacement maps
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
    Alpha,
}

/// Composite operators
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum CompositeOperator {
    Over,
    In,
    Out,
    Atop,
    Xor,
    Arithmetic { k1: f32, k2: f32, k3: f32, k4: f32 },
}

/// Morphology operators
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MorphologyOperator {
    Erode,
    Dilate,
}

impl Filter {
    /// Create a new filter with the given operations
    pub fn new(shapes: Vec<Shape>, operations: Vec<FilterOperation>) -> Self {
        Self { shapes, operations }
    }

    /// Create a gaussian blur filter
    pub fn gaussian_blur(shapes: Vec<Shape>, std_dev: f32) -> Self {
        Self {
            shapes,
            operations: vec![FilterOperation::GaussianBlur {
                std_dev_x: std_dev,
                std_dev_y: std_dev
            }],
        }
    }

    // Add other convenience constructors
}

// Add Filter to the Shape enum
impl From<Filter> for Shape {
    #[inline(always)]
    fn from(filter: Filter) -> Self {
        Self::Filter(filter)
    }
}

// Update Shape enum
pub enum Shape {
    // Existing variants...

    /// A filter applied to shapes
    Filter(Filter),
}
```

## 9. Integration with Shape

```rust
// Update the Shape enum to include all new shape types
pub enum Shape {
    /// Paint nothing. This can be useful as a placeholder.
    Noop,

    /// Recursively nest more shapes - sometimes a convenience to be able to do.
    /// For performance reasons it is better to avoid it.
    Vec(Vec<Shape>),

    /// Circle with optional outline and fill.
    Circle(CircleShape),

    /// Ellipse with optional outline and fill.
    Ellipse(EllipseShape),

    /// A line between two points.
    LineSegment { points: [Pos2; 2], stroke: Stroke },

    /// A series of lines between points.
    /// The path can have a stroke and/or fill (if closed).
    Path(PathShape),

    /// Rectangle with optional outline and fill.
    Rect(RectShape),

    /// Text.
    ///
    /// This needs to be recreated if `pixels_per_point` (dpi scale) changes.
    Text(TextShape),

    /// A general triangle mesh.
    ///
    /// Can be used to display images.
    ///
    /// Wrapped in an [`Arc`] to minimize the size of [`Shape`].
    Mesh(Arc<Mesh>),

    /// A quadratic [Bézier Curve](https://en.wikipedia.org/wiki/B%C3%A9zier_curve).
    QuadraticBezier(QuadraticBezierShape),

    /// A cubic [Bézier Curve](https://en.wikipedia.org/wiki/B%C3%A9zier_curve).
    CubicBezier(CubicBezierShape),

    /// An arc segment.
    Arc(ArcShape),

    /// Backend-specific painting.
    Callback(PaintCallback),

    /// A group of shapes with a specific blend mode.
    BlendGroup(BlendGroup),

    /// A masked group of shapes.
    Mask(Mask),

    /// A filter applied to shapes.
    Filter(Filter),
}

// Add constructors for the new shape types
impl Shape {
    /// Create an arc
    #[inline]
    pub fn arc(
        start: Pos2,
        end: Pos2,
        radii: Vec2,
        x_rotation: f32,
        large_arc: bool,
        sweep: bool,
        stroke: impl Into<PathStroke>,
    ) -> Self {
        Self::Arc(ArcShape::new(
            start, end, radii, x_rotation, large_arc, sweep,
            Color32::TRANSPARENT, stroke
        ))
    }

    /// Create a filled arc
    #[inline]
    pub fn arc_filled(
        start: Pos2,
        end: Pos2,
        radii: Vec2,
        x_rotation: f32,
        large_arc: bool,
        sweep: bool,
        fill: impl Into<Color32>,
        stroke: impl Into<PathStroke>,
    ) -> Self {
        Self::Arc(ArcShape::new(
            start, end, radii, x_rotation, large_arc, sweep,
            fill.into(), stroke
        ))
    }

    /// Create a blend group
    #[inline]
    pub fn blend_group(shapes: Vec<Shape>, blend_mode: BlendMode) -> Self {
        Self::BlendGroup(BlendGroup::new(shapes, blend_mode))
    }

    /// Create an alpha mask
    #[inline]
    pub fn alpha_mask(mask_shape: impl Into<Shape>, shapes: Vec<Shape>) -> Self {
        Self::Mask(Mask::alpha(mask_shape, shapes))
    }

    /// Create a gaussian blur filter
    #[inline]
    pub fn blur_filter(shapes: Vec<Shape>, std_dev: f32) -> Self {
        Self::Filter(Filter::gaussian_blur(shapes, std_dev))
    }
}
```

## Implementation Plan

1. **Phase 1: Core Path and Stroke Enhancements**
   - Implement Arc support
   - Enhance Bézier curves with smooth capabilities
   - Enhance Stroke with line cap, join, and miter limit

2. **Phase 2: Fill and Gradient Support**
   - Implement fill rules
   - Add gradient support

3. **Phase 3: Transform Enhancements**
   - Implement full transform matrix

4. **Phase 4: Advanced Effects**
   - Implement text on path
   - Add blend modes
   - Implement masking
   - Add filter effects

## Backward Compatibility

The design ensures backward compatibility by:

1. Keeping all existing structures and their API intact
2. Adding new functionality through extensions and optional fields
3. Ensuring default behavior matches current behavior
4. Providing convenience methods for new capabilities

## Performance Considerations

1. **Memory Efficiency**
   - Use enums and optional fields to minimize memory usage
   - Keep Arc for shared data

2. **Rendering Optimization**
   - Add caching for complex operations like path flattening
   - Implement early-out optimizations for simple cases

3. **GPU-friendly Design**
   - Design filter and blend operations to be GPU-accelerated where possible
   - Optimize gradient computations

## Future Extensions

1. **Animation Support**
   - Add animation properties and interpolation
   - Enable keyframe-based animation

2. **Pattern Support**
   - Add pattern fills with repetition options

3. **User Interaction**
   - Enable hit testing for all shape types

The design provides a comprehensive foundation for SVG rendering while maintaining backward compatibility and performance.
