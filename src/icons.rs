//! Minimal procedural vector icons drawn with `egui` primitives (zero icon-font bloat).

use egui::{Color32, Painter, Pos2, Rect, Stroke};

fn at(rect: Rect, x: f32, y: f32) -> Pos2 {
    egui::pos2(
        rect.left() + x / 24.0 * rect.width(),
        rect.top() + y / 24.0 * rect.height(),
    )
}

fn scale(rect: Rect) -> f32 {
    (rect.width() / 24.0).max(0.5)
}

fn pen(rect: Rect, color: Color32, width: f32) -> Stroke {
    Stroke::new((width * scale(rect)).max(0.8), color)
}

fn ln(painter: &Painter, rect: Rect, a: (f32, f32), b: (f32, f32), stroke: Stroke) {
    painter.line_segment([at(rect, a.0, a.1), at(rect, b.0, b.1)], stroke);
}

fn box_rect(rect: Rect, x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_min_max(at(rect, x0, y0), at(rect, x1, y1))
}

pub fn image(painter: &Painter, rect: Rect, color: Color32) {
    let s = pen(rect, color, 1.8);
    painter.rect_stroke(
        box_rect(rect, 3.0, 4.0, 21.0, 20.0),
        egui::Rounding::same(3.0 * scale(rect)),
        s,
    );
    painter.circle_filled(at(rect, 8.5, 9.5), 1.6 * scale(rect), color);
    ln(painter, rect, (3.8, 17.0), (9.0, 11.5), s);
    ln(painter, rect, (9.0, 11.5), (12.5, 15.0), s);
    ln(painter, rect, (12.5, 15.0), (15.8, 11.8), s);
    ln(painter, rect, (15.8, 11.8), (20.2, 17.0), s);
}

pub fn upload(painter: &Painter, rect: Rect, color: Color32) {
    let s = pen(rect, color, 1.8);
    ln(painter, rect, (12.0, 15.5), (12.0, 3.5), s);
    ln(painter, rect, (7.5, 8.0), (12.0, 3.5), s);
    ln(painter, rect, (16.5, 8.0), (12.0, 3.5), s);
    ln(painter, rect, (3.5, 15.0), (3.5, 20.0), s);
    ln(painter, rect, (3.5, 20.0), (20.5, 20.0), s);
    ln(painter, rect, (20.5, 20.0), (20.5, 15.0), s);
}

pub fn download(painter: &Painter, rect: Rect, color: Color32) {
    let s = pen(rect, color, 1.8);
    ln(painter, rect, (12.0, 3.5), (12.0, 15.0), s);
    ln(painter, rect, (7.5, 10.5), (12.0, 15.0), s);
    ln(painter, rect, (16.5, 10.5), (12.0, 15.0), s);
    ln(painter, rect, (3.5, 16.0), (3.5, 20.0), s);
    ln(painter, rect, (3.5, 20.0), (20.5, 20.0), s);
    ln(painter, rect, (20.5, 20.0), (20.5, 16.0), s);
}

pub fn check(painter: &Painter, rect: Rect, color: Color32) {
    let s = pen(rect, color, 2.2);
    ln(painter, rect, (4.5, 12.5), (9.5, 17.5), s);
    ln(painter, rect, (9.5, 17.5), (19.5, 6.5), s);
}

pub fn close(painter: &Painter, rect: Rect, color: Color32) {
    let s = pen(rect, color, 1.9);
    ln(painter, rect, (5.5, 5.5), (18.5, 18.5), s);
    ln(painter, rect, (18.5, 5.5), (5.5, 18.5), s);
}

pub fn info(painter: &Painter, rect: Rect, color: Color32) {
    let s = pen(rect, color, 1.8);
    painter.circle_stroke(at(rect, 12.0, 12.0), 9.0 * scale(rect), s);
    painter.circle_filled(at(rect, 12.0, 7.8), 1.15 * scale(rect), color);
    ln(painter, rect, (12.0, 11.0), (12.0, 16.5), s);
}

pub fn folder(painter: &Painter, rect: Rect, color: Color32) {
    let s = pen(rect, color, 1.7);
    painter.rect_stroke(
        box_rect(rect, 3.0, 6.0, 21.0, 19.5),
        egui::Rounding::same(2.5 * scale(rect)),
        s,
    );
    ln(painter, rect, (3.0, 10.0), (21.0, 10.0), s);
    ln(painter, rect, (3.0, 6.0), (9.5, 6.0), s);
}

pub fn dashed_round_rect(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    stroke: Stroke,
    dash: f32,
    gap: f32,
) {
    let r = radius
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);
    let steps = 5_i32;
    let corners: [(Pos2, f32, f32); 4] = [
        (egui::pos2(rect.right() - r, rect.top() + r), -90.0, 0.0),
        (egui::pos2(rect.right() - r, rect.bottom() - r), 0.0, 90.0),
        (egui::pos2(rect.left() + r, rect.bottom() - r), 90.0, 180.0),
        (egui::pos2(rect.left() + r, rect.top() + r), 180.0, 270.0),
    ];

    let mut outline: Vec<Pos2> = Vec::with_capacity(4 * (steps as usize + 1) + 1);
    for (center, start, end) in corners {
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let angle = (start + (end - start) * t).to_radians();
            outline.push(egui::pos2(
                center.x + r * angle.cos(),
                center.y + r * angle.sin(),
            ));
        }
    }
    if let Some(&first) = outline.first() {
        outline.push(first);
    }

    for shape in egui::Shape::dashed_line(&outline, stroke, dash, gap) {
        painter.add(shape);
    }
}
