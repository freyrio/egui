#![allow(clippy::derived_hash_with_manual_eq)] // We need to impl Hash for f32, but we don't implement Eq, which is fine

use std::{fmt::Debug, sync::Arc};

use super::{emath, Color32, ColorMode, Pos2, Rect};

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

impl Default for LineCap {
    fn default() -> Self {
        Self::Butt
    }
}

impl Default for LineJoin {
    fn default() -> Self {
        Self::Miter
    }
}

/// Describes the width and color of a line.
///
/// The default stroke is the same as [`Stroke::NONE`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Stroke {
    pub width: f32,
    pub color: Color32,
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

    #[inline]
    pub fn new(width: impl Into<f32>, color: impl Into<Color32>) -> Self {
        Self {
            width: width.into(),
            color: color.into(),
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 4.0,
        }
    }

    /// True if width is zero or color is transparent
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.color == Color32::TRANSPARENT
    }

    /// Set the line cap
    #[inline]
    pub fn with_line_cap(mut self, cap: LineCap) -> Self {
        self.cap = cap;
        self
    }

    /// Set the line join
    #[inline]
    pub fn with_line_join(mut self, join: LineJoin) -> Self {
        self.join = join;
        self
    }

    /// Set the miter limit
    #[inline]
    pub fn with_miter_limit(mut self, miter_limit: f32) -> Self {
        self.miter_limit = miter_limit;
        self
    }
}

impl<Color> From<(f32, Color)> for Stroke
where
    Color: Into<Color32>,
{
    #[inline(always)]
    fn from((width, color): (f32, Color)) -> Self {
        Self::new(width, color)
    }
}

impl std::hash::Hash for Stroke {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self {
            width,
            color,
            cap,
            join,
            miter_limit,
        } = *self;
        emath::OrderedFloat(width).hash(state);
        color.hash(state);
        cap.hash(state);
        join.hash(state);
        emath::OrderedFloat(miter_limit).hash(state);
    }
}

/// Describes how the stroke of a shape should be painted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum StrokeKind {
    /// The stroke should be painted entirely inside of the shape
    Inside,

    /// The stroke should be painted right on the edge of the shape, half inside and half outside.
    Middle,

    /// The stroke should be painted entirely outside of the shape
    Outside,
}

/// Describes the width and color of paths. The color can either be solid or provided by a callback. For more information, see [`ColorMode`]
///
/// The default stroke is the same as [`PathStroke::NONE`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PathStroke {
    pub width: f32,
    pub color: ColorMode,
    pub kind: StrokeKind,
    pub cap: LineCap,
    pub join: LineJoin,
    pub miter_limit: f32,
}

impl Default for PathStroke {
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

impl PathStroke {
    /// Same as [`PathStroke::default`].
    pub const NONE: Self = Self {
        width: 0.0,
        color: ColorMode::TRANSPARENT,
        kind: StrokeKind::Middle,
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        miter_limit: 4.0,
    };

    #[inline]
    pub fn new(width: impl Into<f32>, color: impl Into<Color32>) -> Self {
        Self {
            width: width.into(),
            color: ColorMode::Solid(color.into()),
            kind: StrokeKind::Middle,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 4.0,
        }
    }

    /// Create a new `PathStroke` with a UV function
    ///
    /// The bounding box passed to the callback will have a margin of [`TessellationOptions::feathering_size_in_pixels`](`crate::tessellator::TessellationOptions::feathering_size_in_pixels`)
    #[inline]
    pub fn new_uv(
        width: impl Into<f32>,
        callback: impl Fn(Rect, Pos2) -> Color32 + Send + Sync + 'static,
    ) -> Self {
        Self {
            width: width.into(),
            color: ColorMode::UV(Arc::new(callback)),
            kind: StrokeKind::Middle,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 4.0,
        }
    }

    #[inline]
    pub fn with_kind(mut self, kind: StrokeKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set the stroke to be painted right on the edge of the shape, half inside and half outside.
    #[inline]
    pub fn middle(mut self) -> Self {
        self.kind = StrokeKind::Middle;
        self
    }

    /// Set the stroke to be painted entirely outside of the shape
    #[inline]
    pub fn outside(mut self) -> Self {
        self.kind = StrokeKind::Outside;
        self
    }

    /// Set the stroke to be painted entirely inside of the shape
    #[inline]
    pub fn inside(mut self) -> Self {
        self.kind = StrokeKind::Inside;
        self
    }

    /// Set the line cap
    #[inline]
    pub fn with_line_cap(mut self, cap: LineCap) -> Self {
        self.cap = cap;
        self
    }

    /// Set the line join
    #[inline]
    pub fn with_line_join(mut self, join: LineJoin) -> Self {
        self.join = join;
        self
    }

    /// Set the miter limit
    #[inline]
    pub fn with_miter_limit(mut self, miter_limit: f32) -> Self {
        self.miter_limit = miter_limit;
        self
    }

    /// True if width is zero or color is solid and transparent
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.color == ColorMode::TRANSPARENT
    }
}

impl<Color> From<(f32, Color)> for PathStroke
where
    Color: Into<Color32>,
{
    #[inline(always)]
    fn from((width, color): (f32, Color)) -> Self {
        Self::new(width, color)
    }
}

impl From<Stroke> for PathStroke {
    fn from(value: Stroke) -> Self {
        if value.is_empty() {
            // Important, since we use the stroke color when doing feathering of the fill!
            Self::NONE
        } else {
            Self {
                width: value.width,
                color: ColorMode::Solid(value.color),
                kind: StrokeKind::Middle,
                cap: value.cap,
                join: value.join,
                miter_limit: value.miter_limit,
            }
        }
    }
}

impl std::hash::Hash for PathStroke {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        emath::OrderedFloat(self.width).hash(state);
        // Skip hashing self.color since UV variant contains a closure
        self.kind.hash(state);
        self.cap.hash(state);
        self.join.hash(state);
        emath::OrderedFloat(self.miter_limit).hash(state);
    }
}
