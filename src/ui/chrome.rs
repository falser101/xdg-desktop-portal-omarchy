//! Shared Apple-like chrome: filled primary actions, pill search, sheets.

use egui::{
    Align, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Order, Pos2, Rect,
    RichText, Sense, Stroke, Vec2,
};

pub const TITLE_PT: f32 = 17.0;
pub const BODY_PT: f32 = 13.0;
pub const CAPTION_PT: f32 = 11.0;
pub const BUTTON_H: f32 = 32.0;
pub const BUTTON_MIN_W: f32 = 82.0;
pub const BUTTON_R: u8 = 8;
pub const ROW_H: f32 = 32.0;
pub const ROW_ICON: f32 = 20.0;
pub const SIDEBAR_W: f32 = 200.0;
pub const TOOLBAR_BTN: f32 = 28.0;

pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

pub fn title_text(text: impl Into<String>) -> RichText {
    RichText::new(text).family(semibold()).size(TITLE_PT)
}

pub fn body_text(text: impl Into<String>) -> RichText {
    RichText::new(text).size(BODY_PT)
}

pub fn caption_text(text: impl Into<String>, color: Color32) -> RichText {
    RichText::new(text).size(CAPTION_PT).color(color)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_unmultiplied(
        lerp_u8(a.r(), b.r(), t),
        lerp_u8(a.g(), b.g(), t),
        lerp_u8(a.b(), b.b(), t),
        lerp_u8(a.a(), b.a(), t),
    )
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

pub fn luma(c: Color32) -> f32 {
    0.2126 * (c.r() as f32) + 0.7152 * (c.g() as f32) + 0.0722 * (c.b() as f32)
}

/// Text on a filled accent control: white on dark tints, near-black on light ones.
pub fn on_accent(accent: Color32) -> Color32 {
    if luma(accent) > 160.0 {
        Color32::from_rgb(28, 28, 30)
    } else {
        Color32::WHITE
    }
}

pub fn accent_of(ui: &egui::Ui) -> Color32 {
    ui.visuals().hyperlink_color
}

pub fn muted_of(ui: &egui::Ui) -> Color32 {
    ui.visuals().weak_text_color()
}

pub fn primary_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let accent = accent_of(ui);
    let text = RichText::new(label)
        .family(semibold())
        .size(BODY_PT)
        .color(on_accent(accent));
    let btn = egui::Button::new(text)
        .fill(accent)
        .stroke(Stroke::NONE)
        .corner_radius(BUTTON_R)
        .min_size(Vec2::new(BUTTON_MIN_W, BUTTON_H));
    ui.add(btn)
}

pub fn secondary_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let fill = ui.visuals().widgets.inactive.bg_fill;
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    let btn = egui::Button::new(RichText::new(label).size(BODY_PT))
        .fill(fill)
        .stroke(stroke)
        .corner_radius(BUTTON_R)
        .min_size(Vec2::new(BUTTON_MIN_W, BUTTON_H));
    ui.add(btn)
}

pub fn destructive_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let fill = ui.visuals().error_fg_color;
    let text = RichText::new(label)
        .family(semibold())
        .size(BODY_PT)
        .color(on_accent(fill));
    let btn = egui::Button::new(text)
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(BUTTON_R)
        .min_size(Vec2::new(BUTTON_MIN_W, BUTTON_H));
    ui.add(btn)
}

/// Cancel on the inner side, primary on the trailing edge (macOS sheet order).
pub fn trailing_actions(ui: &mut egui::Ui, cancel: &str, primary: &str) -> (bool, bool) {
    let mut cancelled = false;
    let mut accepted = false;
    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
        accepted = primary_button(ui, primary).clicked();
        ui.add_space(8.0);
        cancelled = secondary_button(ui, cancel).clicked();
    });
    (cancelled, accepted)
}

fn well_stroke(ui: &egui::Ui) -> Stroke {
    Stroke::new(1.0_f32, ui.visuals().widgets.noninteractive.bg_stroke.color)
}

fn well_frame(ui: &egui::Ui) -> Frame {
    Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(well_stroke(ui))
        .corner_radius(CornerRadius::same(BUTTON_R))
        .inner_margin(Margin::symmetric(12, 0))
}

pub fn search_field(
    ui: &mut egui::Ui,
    query: &mut String,
    hint: &str,
    width: f32,
) -> egui::Response {
    well_frame(ui)
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new((width - 24.0).max(72.0), BUTTON_H));
            ui.set_max_height(BUTTON_H);
            ui.add(
                egui::TextEdit::singleline(query)
                    .desired_width(f32::INFINITY)
                    .hint_text(RichText::new(hint).size(BODY_PT).color(muted_of(ui)))
                    .font(FontId::proportional(BODY_PT))
                    .margin(Margin::symmetric(0, 8))
                    .frame(false),
            )
        })
        .inner
}

pub fn well_edit(ui: &mut egui::Ui, text: &mut String, hint: &str, width: f32) -> egui::Response {
    well_frame(ui)
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(width.max(72.0), BUTTON_H));
            ui.set_max_height(BUTTON_H);
            ui.add(
                egui::TextEdit::singleline(text)
                    .desired_width(width)
                    .hint_text(RichText::new(hint).size(BODY_PT).color(muted_of(ui)))
                    .font(FontId::proportional(BODY_PT))
                    .margin(Margin::symmetric(0, 8))
                    .frame(false),
            )
        })
        .inner
}

/// iOS/macOS-style switch. Returns true when the value changes.
pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> bool {
    let size = Vec2::new(46.0, 28.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let changed = resp.clicked();
    if changed {
        *on = !*on;
    }
    let t = ui.ctx().animate_bool_responsive(resp.id, *on);
    let off = mix(
        ui.visuals().extreme_bg_color,
        ui.visuals().text_color(),
        0.22,
    );
    let fill = mix(off, accent_of(ui), t);
    ui.painter()
        .rect_filled(rect, CornerRadius::same(14), fill);
    let r = rect.height() * 0.5 - 2.5;
    let x = rect.left() + 2.5 + r + t * (rect.width() - 5.0 - 2.0 * r);
    ui.painter()
        .circle_filled(Pos2::new(x, rect.center().y), r, Color32::WHITE);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    changed
}

pub fn labeled_toggle(ui: &mut egui::Ui, on: &mut bool, label: &str) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.set_min_height(BUTTON_H);
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.label(body_text(label).color(ui.visuals().text_color()));
        changed = toggle_switch(ui, on);
    });
    changed
}

pub fn toolbar_glyph_button(ui: &mut egui::Ui, glyph: super::glyphs::Glyph) -> bool {
    let size = Vec2::splat(TOOLBAR_BTN);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();
    let fill = if hovered {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(8), fill);
    let icon = Rect::from_center_size(rect.center(), Vec2::splat(16.0));
    super::glyphs::paint(ui, icon, glyph, ui.visuals().text_color());
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn sidebar_item(
    ui: &mut egui::Ui,
    glyph: super::glyphs::Glyph,
    label: &str,
    selected: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), ROW_H),
        Sense::click(),
    );
    let hovered = resp.hovered();
    let fill = if selected {
        ui.visuals().selection.bg_fill
    } else if hovered {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    let pad = Rect::from_min_max(
        Pos2::new(rect.left() + 4.0, rect.top() + 1.0),
        Pos2::new(rect.right() - 4.0, rect.bottom() - 1.0),
    );
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(pad, CornerRadius::same(8), fill);
    }
    let icon = Rect::from_center_size(
        Pos2::new(pad.left() + 16.0, pad.center().y),
        Vec2::splat(18.0),
    );
    let fg = if selected {
        ui.visuals().strong_text_color()
    } else {
        ui.visuals().text_color()
    };
    super::glyphs::paint(ui, icon, glyph, fg);
    ui.painter().text(
        Pos2::new(icon.right() + 8.0, pad.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::new(BODY_PT, if selected { semibold() } else { FontFamily::Proportional }),
        fg,
    );
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(text.to_uppercase())
            .size(11.0)
            .color(muted_of(ui)),
    );
    ui.add_space(2.0);
}

pub fn dim_overlay(ctx: &egui::Context) {
    let rect = ctx.screen_rect();
    let color = if ctx.style().visuals.dark_mode {
        Color32::from_black_alpha(150)
    } else {
        Color32::from_black_alpha(90)
    };
    egui::Area::new(egui::Id::new("portal-dim"))
        .fixed_pos(rect.min)
        .order(Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            ui.allocate_response(rect.size(), Sense::click());
            ui.painter().rect_filled(rect, 0.0, color);
        });
}

pub fn sheet_frame(ctx: &egui::Context) -> Frame {
    let v = &ctx.style().visuals;
    Frame::new()
        .fill(v.window_fill)
        .stroke(v.window_stroke)
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(18))
        .shadow(v.popup_shadow)
}

pub fn hairline(ui: &mut egui::Ui) {
    let y = ui.cursor().top();
    let rect = Rect::from_min_max(
        Pos2::new(ui.max_rect().left(), y),
        Pos2::new(ui.max_rect().right(), y + 1.0),
    );
    ui.painter().rect_filled(
        rect,
        0.0,
        ui.visuals().widgets.noninteractive.bg_stroke.color,
    );
    ui.add_space(1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_accent_gets_dark_label() {
        assert_eq!(
            on_accent(Color32::from_rgb(0xa8, 0xd4, 0xff)),
            Color32::from_rgb(28, 28, 30)
        );
    }

    #[test]
    fn dark_accent_gets_white_label() {
        assert_eq!(on_accent(Color32::from_rgb(0x0a, 0x5f, 0xd6)), Color32::WHITE);
    }

    #[test]
    fn mix_midpoint() {
        let c = mix(Color32::BLACK, Color32::WHITE, 0.5);
        assert_eq!(c.r(), 128);
    }
}
