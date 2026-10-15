//! 颜色、字体和几个用矢量画的小图标。egui 自带字体没有中文，这里嵌入 Noto Sans SC 子集。

use std::borrow::Cow;
use std::sync::Arc;

use egui::{
    Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak,
    Painter, Pos2, Rect, Shape, Stroke, Vec2, pos2, vec2,
};

pub const BLUE: Color32 = Color32::from_rgb(0x2F, 0x6F, 0xED);
pub const GREEN: Color32 = Color32::from_rgb(0x1F, 0xA4, 0x63);
pub const ORANGE: Color32 = Color32::from_rgb(0xF0, 0x7B, 0x1D);
pub const NEUTRAL: Color32 = Color32::from_rgb(0x5B, 0x62, 0x70);
pub const BG: Color32 = Color32::from_rgb(0xF5, 0xF6, 0xF8);
pub const SIDE: Color32 = Color32::from_rgb(0xFB, 0xFB, 0xFC);
pub const TITLEBAR: Color32 = Color32::from_rgb(0xEC, 0xEE, 0xF1);
pub const LINE: Color32 = Color32::from_rgb(0xE6, 0xE8, 0xEC);
pub const TEXT: Color32 = Color32::from_rgb(0x1F, 0x23, 0x29);
pub const SUB: Color32 = Color32::from_rgb(0x8A, 0x90, 0x99);
pub const NAV_ON: Color32 = Color32::from_rgb(0xE8, 0xEF, 0xFE);
pub const NAV_TEXT: Color32 = Color32::from_rgb(0x3A, 0x3F, 0x47);
pub const HEADER_BG: Color32 = Color32::from_rgb(0xFA, 0xFB, 0xFC);
pub const ROW_LINE: Color32 = Color32::from_rgb(0xF0, 0xF1, 0xF3);
pub const HOVER: Color32 = Color32::from_rgb(0xF4, 0xF6, 0xF9);
pub const CANVAS: Color32 = Color32::from_rgb(0xE9, 0xEB, 0xEE);
pub const CALLOUT_BG: Color32 = Color32::from_rgb(0xFF, 0xF4, 0xE5);
pub const CALLOUT_FG: Color32 = Color32::from_rgb(0xB3, 0x5C, 0x00);
pub const CALLOUT_LINE: Color32 = Color32::from_rgb(0xF0, 0xA0, 0x4B);
pub const WHITE: Color32 = Color32::WHITE;

pub const TRAFFIC_GUTTER: f32 = 78.0;
pub const TAB_BAR_H: f32 = 40.0;
pub const SIDEBAR_W: f32 = 220.0;

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
    v.faint_bg_color = HEADER_BG;
    v.code_bg_color = WHITE;
    v.hyperlink_color = BLUE;
    v.selection.bg_fill = Color32::from_rgb(0xD6, 0xE4, 0xFF);
    v.selection.stroke = Stroke::new(1.0, BLUE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, LINE);
    v.widgets.inactive.bg_fill = WHITE;
    v.widgets.inactive.weak_bg_fill = HOVER;
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, BLUE);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0xF3, 0xF6, 0xFD);
    v.widgets.hovered.weak_bg_fill = NAV_ON;
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.active.bg_stroke = Stroke::new(1.0, BLUE);
    v.widgets.active.bg_fill = NAV_ON;
    v.widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.open.bg_stroke = Stroke::new(1.0, LINE);
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

pub fn paint_badge(
    painter: &Painter,
    center: Pos2,
    size: f32,
    radius: u8,
    color: Color32,
    letter: &str,
    font: f32,
) {
    let rect = Rect::from_center_size(center, Vec2::splat(size));
    painter.rect_filled(rect, CornerRadius::same(radius), color);
    if !letter.is_empty() {
        painter.text(
            center,
            Align2::CENTER_CENTER,
            letter,
            FontId::proportional(font),
            WHITE,
        );
    }
}

pub enum NavIcon {
    Home,
    Recent,
    Settings,
}

pub fn paint_nav_icon(painter: &Painter, rect: Rect, icon: NavIcon, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    let x = rect.left();
    let y = rect.top();
    match icon {
        NavIcon::Home => {
            painter.line_segment([pos2(x + 1.5, y + 8.0), pos2(x + 9.0, y + 1.5)], stroke);
            painter.line_segment([pos2(x + 9.0, y + 1.5), pos2(x + 16.5, y + 8.0)], stroke);
            painter.add(Shape::closed_line(
                vec![
                    pos2(x + 4.0, y + 7.5),
                    pos2(x + 4.0, y + 16.0),
                    pos2(x + 14.0, y + 16.0),
                    pos2(x + 14.0, y + 7.5),
                ],
                stroke,
            ));
            painter.add(Shape::closed_line(
                vec![
                    pos2(x + 7.2, y + 16.0),
                    pos2(x + 7.2, y + 11.0),
                    pos2(x + 10.8, y + 11.0),
                    pos2(x + 10.8, y + 16.0),
                ],
                stroke,
            ));
        }
        NavIcon::Recent => {
            let c = rect.center();
            painter.circle_stroke(c, 7.0, stroke);
            painter.line_segment([c, c + vec2(0.0, -4.2)], stroke);
            painter.line_segment([c, c + vec2(3.4, 2.2)], stroke);
        }
        NavIcon::Settings => {
            let c = rect.center();
            painter.circle_stroke(c, 3.2, stroke);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let dir = vec2(angle.cos(), angle.sin());
                painter.line_segment([c + dir * 5.0, c + dir * 7.4], stroke);
            }
        }
    }
}

pub fn paint_search(painter: &Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    let c = center + vec2(-3.0, -2.0);
    painter.circle_stroke(c, 5.5, stroke);
    painter.line_segment(
        [c + vec2(4.0, 4.0), c + vec2(8.0, 8.0)],
        Stroke::new(1.7, color),
    );
}

pub fn paint_arrow_ne(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(2.2, color);
    let a = pos2(
        rect.left() + rect.width() * 0.28,
        rect.bottom() - rect.height() * 0.28,
    );
    let b = pos2(
        rect.right() - rect.width() * 0.26,
        rect.top() + rect.height() * 0.26,
    );
    painter.line_segment([a, b], stroke);
    painter.line_segment([b, b + vec2(-8.0, 0.0)], stroke);
    painter.line_segment([b, b + vec2(0.0, 8.0)], stroke);
}

pub fn paint_plus(painter: &Painter, center: Pos2, len: f32, thickness: f32, color: Color32) {
    let stroke = Stroke::new(thickness, color);
    painter.line_segment([center + vec2(-len, 0.0), center + vec2(len, 0.0)], stroke);
    painter.line_segment([center + vec2(0.0, -len), center + vec2(0.0, len)], stroke);
}
