//! 浅色定稿的颜色、字体，以及用矢量画的类型图标。egui 自带字体没有中文，这里嵌入 Noto Sans SC 子集。

use std::borrow::Cow;
use std::sync::Arc;

use egui::{
    Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak,
    Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2, pos2, vec2,
};

pub const ACCENT: Color32 = Color32::from_rgb(0x14, 0xB8, 0xA6);
pub const ACCENT_DEEP: Color32 = Color32::from_rgb(0x0D, 0x94, 0x88);
pub const ACCENT_2: Color32 = Color32::from_rgb(0x0E, 0xA5, 0xE9);
pub const DOC: Color32 = ACCENT_DEEP;
pub const SHEET: Color32 = Color32::from_rgb(0x02, 0x84, 0xC7);
pub const DECK: Color32 = Color32::from_rgb(0x63, 0x66, 0xF1);
pub const DECK_INK: Color32 = Color32::from_rgb(0x4F, 0x46, 0xE5);

pub const BG: Color32 = Color32::from_rgb(0xF0, 0xF2, 0xF5);
pub const SIDE: Color32 = Color32::from_rgb(0xFA, 0xFB, 0xFC);
pub const WHITE: Color32 = Color32::WHITE;
pub const ELEV: Color32 = Color32::from_rgb(0xF0, 0xF3, 0xF6);
pub const HOVER: Color32 = Color32::from_rgb(0xF3, 0xF5, 0xF7);
pub const TITLEBAR: Color32 = Color32::from_rgb(0xEC, 0xEF, 0xF3);
pub const BORDER: Color32 = Color32::from_rgb(0xE3, 0xE7, 0xEE);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(0xEE, 0xF1, 0xF5);
pub const TEXT: Color32 = Color32::from_rgb(0x1A, 0x1F, 0x28);
pub const TEXT_2: Color32 = Color32::from_rgb(0x3D, 0x45, 0x54);
pub const TEXT_3: Color32 = Color32::from_rgb(0x6B, 0x73, 0x85);
pub const TEXT_4: Color32 = Color32::from_rgb(0x8B, 0x93, 0xA3);
pub const TAB_IDLE_BG: Color32 = Color32::from_rgb(0xE5, 0xE7, 0xEB);
pub const TAB_IDLE_FG: Color32 = Color32::from_rgb(0x37, 0x41, 0x51);
pub const TAB_IDLE_LINE: Color32 = Color32::from_rgb(0xD1, 0xD5, 0xDB);
pub const TAB_ON_FG: Color32 = Color32::from_rgb(0x11, 0x18, 0x27);
pub const TAB_ON_LINE: Color32 = Color32::from_rgb(0xE5, 0xE7, 0xEB);
pub const TAG_BG: Color32 = Color32::from_rgb(0xF0, 0xF3, 0xF6);
pub const TAG_FG: Color32 = Color32::from_rgb(0x6B, 0x73, 0x85);
pub const CLOSE_X: Color32 = Color32::from_rgb(0x6B, 0x72, 0x80);
pub const NAV_ON: Color32 = Color32::from_rgba_unmultiplied_const(0x0D, 0x94, 0x88, 31);
pub const ACCENT_SOFT: Color32 = Color32::from_rgba_unmultiplied_const(0x14, 0xB8, 0xA6, 31);
pub const CHIP_LINE: Color32 = Color32::from_rgba_unmultiplied_const(0x14, 0xB8, 0xA6, 89);

pub const TRAFFIC_GUTTER: f32 = 84.0;
pub const TAB_BAR_H: f32 = 44.0;
pub const SIDEBAR_W: f32 = 232.0;

pub fn tint(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn cjk_font_bytes() -> &'static [u8] {
    include_bytes!("../assets/fonts/NotoSansSC-Regular.subset.otf")
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        Arc::new(FontData {
            font: Cow::Borrowed(cjk_font_bytes()),
            index: 0,
            tweak: FontTweak::default(),
        }),
    );
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "noto_sans_sc".to_owned());
    }
    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|style| {
        style
            .text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(12.0));
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, FontId::proportional(13.0));
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(10.0, 4.0);
        style.visuals = visuals();
    });
}

pub fn visuals() -> egui::Visuals {
    let mut v = egui::Visuals::light();
    v.panel_fill = BG;
    v.window_fill = WHITE;
    v.extreme_bg_color = WHITE;
    v.faint_bg_color = ELEV;
    v.code_bg_color = WHITE;
    v.hyperlink_color = ACCENT_DEEP;
    v.selection.bg_fill = ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.bg_fill = WHITE;
    v.widgets.inactive.weak_bg_fill = HOVER;
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.hovered.bg_fill = HOVER;
    v.widgets.hovered.weak_bg_fill = ACCENT_SOFT;
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_DEEP);
    v.widgets.active.bg_fill = ACCENT_SOFT;
    v.widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.open.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.open.bg_fill = WHITE;
    let radius = CornerRadius::same(8);
    v.widgets.noninteractive.corner_radius = radius;
    v.widgets.inactive.corner_radius = radius;
    v.widgets.hovered.corner_radius = radius;
    v.widgets.active.corner_radius = radius;
    v.widgets.open.corner_radius = radius;
    v
}

pub fn measure(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), FontId::proportional(size), WHITE)
        .size()
        .x
}

pub fn fit_text(ui: &egui::Ui, text: &str, size: f32, max_width: f32) -> String {
    if measure(ui, text, size) <= max_width {
        return text.to_owned();
    }
    let mut owned: String = text.chars().collect();
    while !owned.is_empty() && measure(ui, &format!("{owned}…"), size) > max_width {
        owned.pop();
    }
    format!("{owned}…")
}

pub struct MarkSpec {
    pub badge: f32,
    pub radius: u8,
    pub fill: Color32,
    pub ink: Color32,
    pub mark: Mark,
    pub glyph: f32,
}

pub fn paint_type_icon(painter: &Painter, center: Pos2, spec: &MarkSpec) {
    let rect = Rect::from_center_size(center, Vec2::splat(spec.badge));
    painter.rect_filled(rect, CornerRadius::same(spec.radius), spec.fill);
    let glyph = Rect::from_center_size(center, Vec2::splat(spec.glyph));
    paint_mark(painter, glyph, spec.mark, spec.ink);
}

#[derive(Clone, Copy)]
pub enum Mark {
    Home,
    Doc,
    Sheet,
    Deck,
    Folder,
    Clock,
    Gear,
    Search,
    Refresh,
    List,
    Grid,
}

pub fn paint_mark(painter: &Painter, rect: Rect, mark: Mark, color: Color32) {
    let stroke = mark_stroke(rect, color);
    match mark {
        Mark::Home => {
            painter.line_segment([at(rect, 4.0, 11.0), at(rect, 12.0, 4.0)], stroke);
            painter.line_segment([at(rect, 12.0, 4.0), at(rect, 20.0, 11.0)], stroke);
            painter.add(Shape::closed_line(
                vec![
                    at(rect, 6.0, 10.0),
                    at(rect, 6.0, 19.0),
                    at(rect, 18.0, 19.0),
                    at(rect, 18.0, 10.0),
                ],
                stroke,
            ));
        }
        Mark::Doc => {
            painter.add(Shape::closed_line(
                vec![
                    at(rect, 6.0, 3.0),
                    at(rect, 14.0, 3.0),
                    at(rect, 19.0, 8.0),
                    at(rect, 19.0, 20.0),
                    at(rect, 6.0, 20.0),
                ],
                stroke,
            ));
            painter.line_segment([at(rect, 14.0, 3.0), at(rect, 14.0, 8.0)], stroke);
            painter.line_segment([at(rect, 14.0, 8.0), at(rect, 19.0, 8.0)], stroke);
            painter.line_segment([at(rect, 8.0, 13.0), at(rect, 16.0, 13.0)], stroke);
            painter.line_segment([at(rect, 8.0, 17.0), at(rect, 13.0, 17.0)], stroke);
        }
        Mark::Sheet => {
            let body = Rect::from_min_max(at(rect, 4.0, 4.0), at(rect, 20.0, 20.0));
            painter.rect_stroke(body, CornerRadius::same(2), stroke, StrokeKind::Inside);
            for y in [10.0, 16.0] {
                painter.line_segment([at(rect, 4.0, y), at(rect, 20.0, y)], stroke);
            }
            for x in [10.0, 16.0] {
                painter.line_segment([at(rect, x, 4.0), at(rect, x, 20.0)], stroke);
            }
        }
        Mark::Deck => {
            let body = Rect::from_min_max(at(rect, 3.0, 5.0), at(rect, 21.0, 19.0));
            painter.rect_stroke(body, CornerRadius::same(2), stroke, StrokeKind::Inside);
            painter.add(Shape::convex_polygon(
                vec![
                    at(rect, 10.0, 9.0),
                    at(rect, 15.0, 12.0),
                    at(rect, 10.0, 15.0),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Mark::Folder => {
            painter.add(Shape::closed_line(
                vec![
                    at(rect, 3.0, 9.0),
                    at(rect, 3.0, 19.0),
                    at(rect, 21.0, 19.0),
                    at(rect, 21.0, 9.0),
                    at(rect, 12.0, 9.0),
                    at(rect, 10.0, 7.0),
                    at(rect, 5.0, 7.0),
                ],
                stroke,
            ));
        }
        Mark::Clock => {
            let unit = rect.width() / 24.0;
            let center = rect.center();
            painter.circle_stroke(center, 9.0 * unit, stroke);
            painter.line_segment([center, center + vec2(0.0, -5.0 * unit)], stroke);
            painter.line_segment([center, center + vec2(3.0 * unit, 2.0 * unit)], stroke);
        }
        Mark::Gear => {
            let unit = rect.width() / 24.0;
            let center = rect.center();
            painter.circle_stroke(center, 3.0 * unit, stroke);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let dir = vec2(angle.cos(), angle.sin());
                painter.line_segment(
                    [center + dir * 5.2 * unit, center + dir * 8.4 * unit],
                    stroke,
                );
            }
        }
        Mark::Search => {
            let unit = rect.width() / 24.0;
            painter.circle_stroke(at(rect, 11.0, 11.0), 7.0 * unit, stroke);
            painter.line_segment([at(rect, 16.2, 16.2), at(rect, 20.0, 20.0)], stroke);
        }
        Mark::Refresh => {
            let unit = rect.width() / 24.0;
            painter.circle_stroke(at(rect, 12.0, 13.0), 7.2 * unit, stroke);
            let tip = at(rect, 20.0, 6.0);
            painter.line_segment([at(rect, 15.2, 5.0), tip], stroke);
            painter.line_segment([tip, at(rect, 19.0, 10.2)], stroke);
        }
        Mark::List => {
            let dot = (rect.width() / 24.0).max(1.0);
            for y in [6.0, 12.0, 18.0] {
                painter.circle_filled(at(rect, 3.5, y), dot * 0.7, color);
                painter.line_segment([at(rect, 8.0, y), at(rect, 21.0, y)], stroke);
            }
        }
        Mark::Grid => {
            for (x, y) in [(3.0, 3.0), (14.0, 3.0), (3.0, 14.0), (14.0, 14.0)] {
                let cell = Rect::from_min_max(at(rect, x, y), at(rect, x + 7.0, y + 7.0));
                painter.rect_stroke(cell, CornerRadius::same(1), stroke, StrokeKind::Inside);
            }
        }
    }
}

pub fn paint_avatar(painter: &Painter, center: Pos2) {
    let radius = 13.0;
    let bounds = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    let steps = 8;
    for y in 0..steps {
        for x in 0..steps {
            let t = (x + y) as f32 / ((steps - 1) * 2) as f32;
            let cell = Rect::from_min_max(
                pos2(
                    bounds.left() + bounds.width() * x as f32 / steps as f32,
                    bounds.top() + bounds.height() * y as f32 / steps as f32,
                ),
                pos2(
                    bounds.left() + bounds.width() * (x + 1) as f32 / steps as f32,
                    bounds.top() + bounds.height() * (y + 1) as f32 / steps as f32,
                ),
            );
            painter
                .with_clip_rect(cell)
                .circle_filled(center, radius, lerp(ACCENT, ACCENT_2, t));
        }
    }
    painter.text(
        center,
        Align2::CENTER_CENTER,
        "墨",
        FontId::proportional(11.0),
        WHITE,
    );
}

pub fn paint_plus(painter: &Painter, center: Pos2, len: f32, thickness: f32, color: Color32) {
    let stroke = Stroke::new(thickness, color);
    painter.line_segment([center + vec2(-len, 0.0), center + vec2(len, 0.0)], stroke);
    painter.line_segment([center + vec2(0.0, -len), center + vec2(0.0, len)], stroke);
}

fn at(rect: Rect, x: f32, y: f32) -> Pos2 {
    pos2(
        rect.left() + rect.width() * (x / 24.0),
        rect.top() + rect.height() * (y / 24.0),
    )
}

fn mark_stroke(rect: Rect, color: Color32) -> Stroke {
    Stroke::new((rect.width() * 1.7 / 24.0).clamp(1.05, 1.8), color)
}

fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |from: u8, to: u8| (from as f32 + (to as f32 - from as f32) * t).round() as u8;
    Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
}
