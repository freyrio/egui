use egui::epaint::{ArcShape, LineCap, LineJoin, PathStroke, StrokeKind};
use egui::{vec2, Color32, Pos2, Vec2};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StrokeAndArcsDemo {
    line_cap: LineCap,
    line_join: LineJoin,
    stroke_width: f32,
    miter_limit: f32,
    stroke_kind: StrokeKind,
}

impl Default for StrokeAndArcsDemo {
    fn default() -> Self {
        Self {
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            stroke_width: 8.0, // Make stroke wider by default to better see the effects
            miter_limit: 4.0,
            stroke_kind: StrokeKind::Middle,
        }
    }
}

impl super::Demo for StrokeAndArcsDemo {
    fn name(&self) -> &'static str {
        "🎨 Strokes & Arcs"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Enhanced Stroke & Arc Demo");
                ui.add_space(8.0);

                // Controls
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Line Cap:");
                        ui.selectable_value(&mut self.line_cap, LineCap::Butt, "Butt");
                        ui.selectable_value(&mut self.line_cap, LineCap::Round, "Round");
                        ui.selectable_value(&mut self.line_cap, LineCap::Square, "Square");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Line Join:");
                        ui.selectable_value(&mut self.line_join, LineJoin::Miter, "Miter");
                        ui.selectable_value(&mut self.line_join, LineJoin::Round, "Round");
                        ui.selectable_value(&mut self.line_join, LineJoin::Bevel, "Bevel");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Stroke Kind:");
                        ui.selectable_value(&mut self.stroke_kind, StrokeKind::Inside, "Inside");
                        ui.selectable_value(&mut self.stroke_kind, StrokeKind::Middle, "Middle");
                        ui.selectable_value(&mut self.stroke_kind, StrokeKind::Outside, "Outside");
                    });

                    ui.add(
                        egui::Slider::new(&mut self.stroke_width, 1.0..=20.0).text("Stroke Width"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.miter_limit, 0.0..=10.0).text("Miter Limit"),
                    );
                });

                // Drawing area
                let (response, painter) =
                    ui.allocate_painter(vec2(ui.available_width(), 200.0), egui::Sense::hover());
                let rect = response.rect;

                // Create a path stroke with current settings
                let path_stroke = PathStroke::new(self.stroke_width, Color32::BLUE)
                    .with_line_cap(self.line_cap)
                    .with_line_join(self.line_join)
                    .with_miter_limit(self.miter_limit)
                    .with_kind(self.stroke_kind);

                // Draw an arc
                let center = rect.center();
                let radius = rect.height() / 4.0;
                let start = Pos2::new(center.x - radius, center.y);
                let end = Pos2::new(center.x + radius, center.y);
                let arc = ArcShape::new(
                    center,
                    start,
                    end,
                    Vec2::splat(radius),
                    0.0,
                    false,
                    true,
                    Color32::TRANSPARENT,
                    path_stroke.clone(),
                );
                painter.add(arc);

                // Draw a zigzag line to demonstrate line joins
                let points = vec![
                    Pos2::new(rect.left() + 20.0, rect.bottom() - 40.0),
                    Pos2::new(rect.left() + 60.0, rect.top() + 40.0),
                    Pos2::new(rect.left() + 100.0, rect.bottom() - 40.0),
                    Pos2::new(rect.left() + 140.0, rect.top() + 40.0),
                    Pos2::new(rect.left() + 180.0, rect.bottom() - 40.0),
                ];

                // Create an open path to show line caps
                let path = egui::epaint::PathShape {
                    points,
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: path_stroke,
                };

                painter.add(egui::Shape::Path(path));

                ui.add_space(8.0);
                ui.label(
                    "The demo shows an arc and a zigzag line with configurable stroke properties.",
                );
            });
    }
}
