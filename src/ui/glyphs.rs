//! Monochrome SF-style glyphs so folders are not theme-tinted yellow icons.

use egui::{Color32, CornerRadius, Pos2, Rect, Shape, Stroke, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Folder,
    Home,
    Downloads,
    Documents,
    Pictures,
    Videos,
    Music,
    Projects,
    Computer,
    Recents,
    ChevronLeft,
}

pub fn for_place(label: &str) -> Glyph {
    match label {
        "Home" => Glyph::Home,
        "Downloads" => Glyph::Downloads,
        "Documents" => Glyph::Documents,
        "Pictures" => Glyph::Pictures,
        "Videos" => Glyph::Videos,
        "Music" => Glyph::Music,
        "Projects" => Glyph::Projects,
        "Computer" => Glyph::Computer,
        "Recents" => Glyph::Recents,
        _ => Glyph::Folder,
    }
}

pub fn paint(ui: &egui::Ui, rect: Rect, glyph: Glyph, color: Color32) {
    let rect = rect.shrink(1.0);
    if rect.width() < 4.0 || rect.height() < 4.0 {
        return;
    }
    match glyph {
        Glyph::Folder | Glyph::Projects => folder(ui, rect, color),
        Glyph::Home => home(ui, rect, color),
        Glyph::Downloads => downloads(ui, rect, color),
        Glyph::Documents => documents(ui, rect, color),
        Glyph::Pictures => pictures(ui, rect, color),
        Glyph::Videos => videos(ui, rect, color),
        Glyph::Music => music(ui, rect, color),
        Glyph::Computer => computer(ui, rect, color),
        Glyph::Recents => recents(ui, rect, color),
        Glyph::ChevronLeft => chevron_left(ui, rect, color),
    }
}

fn folder(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let tab = Rect::from_min_size(
        Pos2::new(rect.left() + rect.width() * 0.06, rect.top() + rect.height() * 0.12),
        Vec2::new(rect.width() * 0.40, rect.height() * 0.26),
    );
    p.rect_filled(
        tab,
        CornerRadius {
            nw: 3,
            ne: 3,
            sw: 0,
            se: 0,
        },
        color,
    );
    let body = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.04, rect.top() + rect.height() * 0.28),
        Pos2::new(rect.right() - rect.width() * 0.04, rect.bottom() - rect.height() * 0.08),
    );
    p.rect_filled(body, CornerRadius::same(3), color);
}

fn home(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let cx = rect.center().x;
    let roof = vec![
        Pos2::new(cx, rect.top() + rect.height() * 0.08),
        Pos2::new(rect.right() - rect.width() * 0.08, rect.top() + rect.height() * 0.48),
        Pos2::new(rect.left() + rect.width() * 0.08, rect.top() + rect.height() * 0.48),
    ];
    p.add(Shape::convex_polygon(roof, color, Stroke::NONE));
    let body = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.22, rect.top() + rect.height() * 0.42),
        Pos2::new(rect.right() - rect.width() * 0.22, rect.bottom() - rect.height() * 0.10),
    );
    p.rect_filled(body, CornerRadius {
        nw: 1,
        ne: 1,
        sw: 2,
        se: 2,
    }, color);
}

fn downloads(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let stroke = Stroke::new(1.7_f32, color);
    let cx = rect.center().x;
    let top = rect.top() + rect.height() * 0.12;
    let mid = rect.top() + rect.height() * 0.58;
    p.line_segment(
        [Pos2::new(cx, top), Pos2::new(cx, mid)],
        stroke,
    );
    let head = vec![
        Pos2::new(cx, rect.top() + rect.height() * 0.72),
        Pos2::new(cx - rect.width() * 0.22, mid - 1.0),
        Pos2::new(cx + rect.width() * 0.22, mid - 1.0),
    ];
    p.add(Shape::convex_polygon(head, color, Stroke::NONE));
    let tray = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.16, rect.bottom() - rect.height() * 0.28),
        Pos2::new(rect.right() - rect.width() * 0.16, rect.bottom() - rect.height() * 0.10),
    );
    p.rect_stroke(tray, CornerRadius::same(2), stroke, egui::StrokeKind::Inside);
}

fn documents(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let page = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.22, rect.top() + rect.height() * 0.08),
        Pos2::new(rect.right() - rect.width() * 0.18, rect.bottom() - rect.height() * 0.08),
    );
    p.rect_filled(page, CornerRadius::same(2), color);
    let well = ui.visuals().panel_fill;
    for i in 0..3 {
        let y = page.top() + page.height() * (0.32 + i as f32 * 0.18);
        p.hline(
            (page.left() + 3.0)..=(page.right() - 3.0),
            y,
            Stroke::new(1.2_f32, well),
        );
    }
}

fn pictures(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let frame = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.10, rect.top() + rect.height() * 0.16),
        Pos2::new(rect.right() - rect.width() * 0.10, rect.bottom() - rect.height() * 0.12),
    );
    p.rect_filled(frame, CornerRadius::same(3), color);
    let well = ui.visuals().panel_fill;
    p.circle_filled(
        Pos2::new(frame.left() + frame.width() * 0.30, frame.top() + frame.height() * 0.32),
        frame.width() * 0.10,
        well,
    );
    let mountain = vec![
        Pos2::new(frame.left() + 2.0, frame.bottom() - 2.0),
        Pos2::new(frame.left() + frame.width() * 0.38, frame.top() + frame.height() * 0.42),
        Pos2::new(frame.left() + frame.width() * 0.58, frame.top() + frame.height() * 0.62),
        Pos2::new(frame.right() - 2.0, frame.bottom() - 2.0),
    ];
    p.add(Shape::convex_polygon(mountain, well, Stroke::NONE));
}

fn videos(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let frame = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.08, rect.top() + rect.height() * 0.20),
        Pos2::new(rect.right() - rect.width() * 0.08, rect.bottom() - rect.height() * 0.20),
    );
    p.rect_filled(frame, CornerRadius::same(3), color);
    let well = ui.visuals().panel_fill;
    let play = vec![
        Pos2::new(frame.left() + frame.width() * 0.38, frame.top() + frame.height() * 0.28),
        Pos2::new(frame.left() + frame.width() * 0.38, frame.bottom() - frame.height() * 0.28),
        Pos2::new(frame.left() + frame.width() * 0.72, frame.center().y),
    ];
    p.add(Shape::convex_polygon(play, well, Stroke::NONE));
}

fn music(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let stem = Rect::from_min_size(
        Pos2::new(rect.center().x + rect.width() * 0.10, rect.top() + rect.height() * 0.12),
        Vec2::new(rect.width() * 0.14, rect.height() * 0.58),
    );
    p.rect_filled(stem, CornerRadius::same(1), color);
    p.circle_filled(
        Pos2::new(rect.center().x - rect.width() * 0.04, rect.bottom() - rect.height() * 0.28),
        rect.width() * 0.20,
        color,
    );
    let beam = Rect::from_min_size(
        Pos2::new(rect.center().x - rect.width() * 0.06, rect.top() + rect.height() * 0.12),
        Vec2::new(rect.width() * 0.32, rect.height() * 0.14),
    );
    p.rect_filled(beam, CornerRadius::same(1), color);
}

fn computer(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let screen = Rect::from_min_max(
        Pos2::new(rect.left() + rect.width() * 0.10, rect.top() + rect.height() * 0.10),
        Pos2::new(rect.right() - rect.width() * 0.10, rect.bottom() - rect.height() * 0.34),
    );
    p.rect_filled(screen, CornerRadius::same(2), color);
    let neck = Rect::from_center_size(
        Pos2::new(rect.center().x, rect.bottom() - rect.height() * 0.28),
        Vec2::new(rect.width() * 0.12, rect.height() * 0.16),
    );
    p.rect_filled(neck, 0.0, color);
    let base = Rect::from_center_size(
        Pos2::new(rect.center().x, rect.bottom() - rect.height() * 0.14),
        Vec2::new(rect.width() * 0.48, rect.height() * 0.10),
    );
    p.rect_filled(base, CornerRadius::same(1), color);
}

fn recents(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.40;
    p.circle_stroke(c, r, Stroke::new(1.7_f32, color));
    p.line_segment(
        [c, Pos2::new(c.x, c.y - r * 0.52)],
        Stroke::new(1.6_f32, color),
    );
    p.line_segment(
        [c, Pos2::new(c.x + r * 0.38, c.y + r * 0.12)],
        Stroke::new(1.6_f32, color),
    );
}

fn chevron_left(ui: &egui::Ui, rect: Rect, color: Color32) {
    let p = ui.painter();
    let c = rect.center();
    let w = rect.width() * 0.16;
    let h = rect.height() * 0.28;
    p.line_segment(
        [Pos2::new(c.x + w, c.y - h), Pos2::new(c.x - w, c.y)],
        Stroke::new(1.8_f32, color),
    );
    p.line_segment(
        [Pos2::new(c.x - w, c.y), Pos2::new(c.x + w, c.y + h)],
        Stroke::new(1.8_f32, color),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_labels_map() {
        assert_eq!(for_place("Home"), Glyph::Home);
        assert_eq!(for_place("Downloads"), Glyph::Downloads);
        assert_eq!(for_place("Recents"), Glyph::Recents);
        assert_eq!(for_place("Work"), Glyph::Folder);
    }
}
