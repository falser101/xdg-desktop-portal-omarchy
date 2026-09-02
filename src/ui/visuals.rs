use crate::theme::OmarchyTheme;
use egui::{Color32, CornerRadius, Stroke, Visuals};

pub fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

pub fn apply(ctx: &egui::Context, theme: &OmarchyTheme) {
    let dark = matches!(theme.mode, crate::theme::ColorScheme::PreferDark);
    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    let bg = rgb(theme.background);
    let panel = rgb(theme.panel);
    let fg = rgb(theme.foreground);
    let muted = rgb(theme.muted);
    let accent = rgb(theme.accent_rgb);

    visuals.panel_fill = bg;
    visuals.window_fill = panel;
    visuals.extreme_bg_color = bg;
    visuals.faint_bg_color = panel;
    visuals.override_text_color = Some(fg);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, muted);
    visuals.widgets.inactive.bg_fill = panel;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, fg);
    visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.35);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, fg);
    visuals.widgets.active.bg_fill = accent.gamma_multiply(0.55);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, fg);
    visuals.selection.bg_fill = accent.gamma_multiply(0.55);
    visuals.selection.stroke = Stroke::new(1.0_f32, accent);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.window_stroke = Stroke::new(1.0_f32, muted);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(14.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(14.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::proportional(18.0),
    );
    ctx.set_style(style);
}
