//! 顶栏：macOS 红绿灯留白、文档标签、搜索和头像。标签在留白右侧，垂直居中。

use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Id, Layout, PointerButton, Rect, Sense,
    Stroke, StrokeKind, TextEdit, Vec2, ViewportCommand, pos2, vec2,
};

use crate::app::Shell;
use crate::editor::Tab;
use crate::kind::DocKind;
use crate::theme::{
    self, ACCENT, BORDER, CLOSE_X, TAB_IDLE_BG, TAB_IDLE_FG, TAB_IDLE_LINE, TAB_ON_FG, TAB_ON_LINE,
    TEXT_4, TITLEBAR, TRAFFIC_GUTTER, WHITE,
};

const TAB_H: f32 = 30.0;
const TAB_GAP: f32 = 2.0;
const SEARCH_W: f32 = 220.0;

pub(crate) fn show_tab_bar(ui: &mut egui::Ui, shell: &mut Shell) {
    let bar = ui.max_rect();
    ui.expand_to_include_rect(bar);
    ui.painter().rect_filled(bar, CornerRadius::ZERO, TITLEBAR);

    let (search_rect, avatar_rect) = chrome_right(bar);
    let drag_rect = Rect::from_min_max(
        pos2(bar.left() + TRAFFIC_GUTTER, bar.top()),
        pos2(search_rect.left() - 4.0, bar.bottom()),
    );
    let drag = ui.interact(drag_rect, Id::new("tabbar-drag"), Sense::click_and_drag());
    if drag.double_clicked() {
        let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    } else if drag.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    #[cfg(not(target_os = "macos"))]
    window_buttons(ui, bar);

    let plus_span = 4.0 + 28.0 + 8.0;
    let tabs_left = bar.left() + TRAFFIC_GUTTER;
    let available = (search_rect.left() - plus_span - tabs_left).max(48.0);
    let widths = tab_widths(ui, shell, available);

    let mut close_at = None;
    let mut x = tabs_left;
    let tab_top = bar.center().y - TAB_H / 2.0;
    for (index, width) in widths.iter().copied().enumerate() {
        let rect = Rect::from_min_size(pos2(x, tab_top), vec2(width, TAB_H));
        match tab_chip(ui, rect, &shell.tabs[index], index == shell.active) {
            TabHit::Select => {
                if index == 0 {
                    shell.go_home();
                } else {
                    shell.active = index;
                }
            }
            TabHit::Close => close_at = Some(index),
            TabHit::None => {}
        }
        x += width + TAB_GAP;
    }
    if !widths.is_empty() {
        x -= TAB_GAP;
    }

    let plus = Rect::from_center_size(pos2(x + 4.0 + 14.0, bar.center().y), Vec2::splat(28.0));
    let plus_response = ui.interact(plus, Id::new("tab-plus"), Sense::click());
    if plus_response.hovered() {
        ui.painter()
            .rect_filled(plus, CornerRadius::same(7), Color32::from_black_alpha(12));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    theme::paint_plus(ui.painter(), plus.center(), 6.0, 1.6, TEXT_4);
    if plus_response.clicked() {
        shell.toggle_menu(plus.left_bottom() + vec2(0.0, 6.0));
    }

    let focus = shell.focus_search;
    chrome_search(ui, search_rect, &mut shell.search, focus);
    shell.focus_search = false;
    theme::paint_avatar(ui.painter(), avatar_rect.center());

    ui.painter()
        .hline(bar.x_range(), bar.bottom(), Stroke::new(1.0, BORDER));

    if let Some(index) = close_at {
        shell.close_tab(index);
    }
}

fn chrome_right(bar: Rect) -> (Rect, Rect) {
    let avatar = Rect::from_center_size(
        pos2(bar.right() - 10.0 - 13.0, bar.center().y),
        Vec2::splat(26.0),
    );
    let room = bar.width() - TRAFFIC_GUTTER - 280.0 - 26.0 - 10.0 - 6.0;
    let search_w = SEARCH_W.min(room.max(140.0));
    let search = Rect::from_min_size(
        pos2(avatar.left() - 6.0 - search_w, bar.center().y - 14.0),
        vec2(search_w, 28.0),
    );
    (search, avatar)
}

enum TabHit {
    None,
    Select,
    Close,
}

fn tab_widths(ui: &egui::Ui, shell: &Shell, available: f32) -> Vec<f32> {
    let natural: Vec<f32> = shell
        .tabs
        .iter()
        .map(|tab| natural_width(ui, tab))
        .collect();
    let gaps = TAB_GAP * natural.len().saturating_sub(1) as f32;
    let sum = natural.iter().sum::<f32>() + gaps;
    if sum <= available || natural.is_empty() {
        return natural;
    }
    let room = (available - gaps).max(0.0);
    let raw = natural.iter().sum::<f32>().max(1.0);
    let scale = room / raw;
    natural.into_iter().map(|width| width * scale).collect()
}

fn natural_width(ui: &egui::Ui, tab: &Tab) -> f32 {
    let text = theme::measure(ui, tab.title(), 12.5);
    let width = if tab.is_home() {
        10.0 + 18.0 + 7.0 + text + 12.0
    } else {
        10.0 + 18.0 + 7.0 + text + 8.0 + 16.0 + 10.0
    };
    width.clamp(108.0, if tab.is_home() { 132.0 } else { 190.0 })
}

fn tab_chip(ui: &mut egui::Ui, rect: Rect, tab: &Tab, selected: bool) -> TabHit {
    let response = ui.interact(
        rect,
        Id::new(("tab", tab.widget_id())),
        Sense::click_and_drag(),
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    let painter = ui.painter();
    let (fill, line) = if selected {
        (WHITE, TAB_ON_LINE)
    } else if response.hovered() {
        (Color32::from_rgb(0xF3, 0xF4, 0xF6), TAB_IDLE_LINE)
    } else {
        (TAB_IDLE_BG, TAB_IDLE_LINE)
    };
    painter.rect_filled(rect, CornerRadius::same(8), fill);
    painter.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0, line),
        StrokeKind::Inside,
    );
    if selected {
        let accent = Rect::from_min_max(
            pos2(rect.left() + 7.0, rect.top() + 1.0),
            pos2(rect.right() - 7.0, rect.top() + 3.0),
        );
        painter.rect_filled(accent, CornerRadius::same(1), ACCENT);
    }

    let (fill_src, ink, mark) = match tab.kind() {
        Some(kind) => (kind.color(), kind.icon_ink(), kind.mark()),
        None => (theme::ACCENT_DEEP, theme::ACCENT_DEEP, theme::Mark::Home),
    };
    theme::paint_type_icon(
        painter,
        pos2(rect.left() + 10.0 + 9.0, rect.center().y),
        &theme::MarkSpec {
            badge: 18.0,
            radius: 4,
            fill: theme::tint(fill_src, 46),
            ink,
            mark,
            glyph: 11.0,
        },
    );

    let text_left = rect.left() + 10.0 + 18.0 + 7.0;
    let dirty = tab.dirty();
    let text_right = if tab.is_home() {
        rect.right() - 10.0
    } else if dirty {
        rect.right() - 42.0
    } else {
        rect.right() - 28.0
    };
    let title = theme::fit_text(ui, tab.title(), 12.5, (text_right - text_left).max(8.0));
    painter.text(
        pos2(text_left, rect.center().y),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(12.5),
        if selected { TAB_ON_FG } else { TAB_IDLE_FG },
    );

    let mut hit = TabHit::None;
    if !tab.is_home() {
        if dirty {
            let dot = pos2(rect.right() - 36.0, rect.center().y);
            painter.circle_filled(dot, 3.0, tab.kind().map(DocKind::color).unwrap_or(ACCENT));
        }
        let close_rect = Rect::from_center_size(
            pos2(rect.right() - 16.0, rect.center().y),
            Vec2::splat(16.0),
        );
        let close = ui.interact(
            close_rect,
            Id::new(("tab-close", tab.widget_id())),
            Sense::click(),
        );
        if close.hovered() {
            painter.rect_filled(
                close_rect,
                CornerRadius::same(4),
                Color32::from_black_alpha(18),
            );
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        painter.text(
            close_rect.center(),
            Align2::CENTER_CENTER,
            "×",
            FontId::proportional(14.0),
            CLOSE_X,
        );
        if close.clicked() {
            hit = TabHit::Close;
        }
    }
    if matches!(hit, TabHit::None) && response.clicked() {
        hit = TabHit::Select;
    }
    hit
}

fn chrome_search(ui: &mut egui::Ui, rect: Rect, text: &mut String, focus: bool) {
    ui.painter().rect_filled(rect, CornerRadius::same(7), WHITE);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(7),
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );
    let icon = Rect::from_center_size(pos2(rect.left() + 16.0, rect.center().y), Vec2::splat(13.0));
    theme::paint_mark(ui.painter(), icon, theme::Mark::Search, TEXT_4);

    let text_rect = Rect::from_min_max(
        pos2(rect.left() + 28.0, rect.top()),
        pos2(rect.right() - 8.0, rect.bottom()),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let output = TextEdit::singleline(text)
        .id(Id::new("home-search"))
        .hint_text("搜索文件…")
        .frame(Frame::NONE)
        .margin(egui::Margin::symmetric(0, 0))
        .desired_width(text_rect.width())
        .font(FontId::proportional(12.5))
        .show(&mut child);
    let response = output.response.response.clone();
    if focus {
        response.request_focus();
    }
    response.on_hover_text(format!("搜索文件（{}）", crate::platform::shortcut("K")));
}

#[cfg(not(target_os = "macos"))]
fn window_buttons(ui: &mut egui::Ui, bar: Rect) {
    let colors = [
        Color32::from_rgb(0xFF, 0x5F, 0x57),
        Color32::from_rgb(0xFE, 0xBC, 0x2E),
        Color32::from_rgb(0x28, 0xC8, 0x40),
    ];
    let cy = bar.center().y;
    for (index, color) in colors.into_iter().enumerate() {
        let center = pos2(bar.left() + 24.0 + index as f32 * 20.0, cy);
        let rect = Rect::from_center_size(center, Vec2::splat(16.0));
        let response = ui.interact(rect, Id::new(("traffic", index)), Sense::click());
        ui.painter().circle_filled(center, 6.0, color);
        if response.hovered() {
            ui.painter().circle_stroke(
                center,
                7.5,
                Stroke::new(1.0, Color32::from_black_alpha(30)),
            );
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if response.clicked() {
            let command = match index {
                0 => ViewportCommand::Close,
                1 => ViewportCommand::Minimized(true),
                _ => {
                    let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                    ViewportCommand::Maximized(!maximized)
                }
            };
            ui.ctx().send_viewport_cmd(command);
        }
    }
}
