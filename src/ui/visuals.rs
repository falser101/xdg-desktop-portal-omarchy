use super::chrome::{self, mix, semibold};
use crate::theme::OmarchyTheme;
use egui::{Color32, CornerRadius, FontId, Margin, Shadow, Stroke, Visuals};

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
    let red = rgb(theme.red);
    chrome::set_type_scale(theme.type_scale());
    let body = chrome::body_pt();
    let caption = chrome::caption_pt();
    let title = chrome::title_pt();

    let well = if dark {
        mix(bg, Color32::BLACK, 0.28)
    } else {
        mix(bg, Color32::WHITE, 0.55)
    };
    let hover = if dark {
        Color32::from_white_alpha(14)
    } else {
        Color32::from_black_alpha(12)
    };
    let inactive_fill = if dark {
        mix(panel, bg, 0.35)
    } else {
        mix(panel, Color32::WHITE, 0.25)
    };
    let hairline = muted.gamma_multiply(if dark { 0.45 } else { 0.35 });
    let selected = rgb(theme.selection);

    visuals.dark_mode = dark;
    visuals.panel_fill = bg;
    visuals.window_fill = bg;
    visuals.extreme_bg_color = well;
    visuals.faint_bg_color = mix(bg, panel, 0.55);
    visuals.override_text_color = Some(fg);
    visuals.hyperlink_color = accent;
    visuals.error_fg_color = red;
    visuals.warn_fg_color = red;

    let hair = Stroke::new(1.0_f32, hairline);
    let fg_stroke = Stroke::new(1.0_f32, fg);

    visuals.widgets.noninteractive.bg_fill = bg;
    visuals.widgets.noninteractive.weak_bg_fill = well;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, muted);
    visuals.widgets.noninteractive.bg_stroke = hair;
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);

    visuals.widgets.inactive.bg_fill = inactive_fill;
    visuals.widgets.inactive.weak_bg_fill = well;
    visuals.widgets.inactive.fg_stroke = fg_stroke;
    visuals.widgets.inactive.bg_stroke = hair;
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

    visuals.widgets.hovered.bg_fill = mix(inactive_fill, hover, 0.65);
    visuals.widgets.hovered.weak_bg_fill = hover;
    visuals.widgets.hovered.fg_stroke = fg_stroke;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, mix(hairline, accent, 0.25));
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);

    visuals.widgets.active.bg_fill = mix(accent, inactive_fill, 0.55);
    visuals.widgets.active.fg_stroke = fg_stroke;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, accent);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);

    visuals.widgets.open.bg_fill = inactive_fill;
    visuals.widgets.open.fg_stroke = fg_stroke;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, accent);
    visuals.widgets.open.corner_radius = CornerRadius::same(8);

    visuals.selection.bg_fill = selected;
    visuals.selection.stroke = Stroke::new(0.0_f32, accent);

    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.window_stroke = Stroke::new(1.0_f32, hairline);
    visuals.window_shadow = Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: if dark {
            Color32::from_black_alpha(140)
        } else {
            Color32::from_black_alpha(36)
        },
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: if dark {
            Color32::from_black_alpha(120)
        } else {
            Color32::from_black_alpha(28)
        },
    };

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.animation_time = 0.16;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.interact_size = egui::vec2(80.0, 32.0);
    style.spacing.window_margin = Margin::same(16);
    style.spacing.menu_margin = Margin::symmetric(6, 6);
    style.spacing.icon_width = 16.0;
    style.spacing.icon_width_inner = 14.0;
    style.text_styles.insert(
        egui::TextStyle::Small,
        FontId::proportional(caption),
    );
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(body));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(body));
    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(title, semibold()),
    );
    ctx.set_style(style);
}
