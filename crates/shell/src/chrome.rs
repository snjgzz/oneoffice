//! 顶栏：macOS 红绿灯留白、首页标签、文档标签、新建按钮。

use egui::{
    Align2, Color32, CornerRadius, FontId, Id, PointerButton, Rect, Sense, Stroke, Vec2,
    ViewportCommand, pos2, vec2,
};

use crate::app::Shell;
use crate::editor::Tab;
use crate::theme::{self, LINE, SUB, TEXT, TRAFFIC_GUTTER, WHITE};

pub(crate) fn show_tab_bar(ui: &mut egui::Ui, shell: &mut Shell) {
    let bar = ui.max_rect();
    ui.expand_to_include_rect(bar);

    let drag_rect = Rect::from_min_max(
        pos2(bar.left() + TRAFFIC_GUTTER, bar.top()),
        bar.right_bottom(),
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

    let mut close_at = None;
    let mut x = bar.left() + TRAFFIC_GUTTER;
    let tab_top = bar.bottom() - 32.0;
    let plus_room = 40.0;
    let available = (bar.right() - plus_room - x).max(120.0);
    let widths = tab_widths(ui, shell, available);

    for (index, width) in widths.into_iter().enumerate() {
        let rect = Rect::from_min_size(pos2(x, tab_top), vec2(width, 32.0));
        match tab_chip(ui, rect, &shell.tabs[index], index == shell.active) {
            TabHit::Select => shell.active = index,
            TabHit::Close => close_at = Some(index),
            TabHit::None => {}
        }
        x += width + 4.0;
    }

    let plus = Rect::from_center_size(pos2(x + 16.0, bar.center().y), Vec2::splat(28.0));
    let plus_response = ui.interact(plus, Id::new("tab-plus"), Sense::click());
    if plus_response.hovered() {
        ui.painter()
            .rect_filled(plus, CornerRadius::same(6), Color32::from_black_alpha(12));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    theme::paint_plus(
        ui.painter(),
        plus.center(),
        6.0,
        1.6,
        Color32::from_rgb(0x77, 0x77, 0x77),
    );
    if plus_response.clicked() {
        shell.toggle_menu(plus.left_bottom() + vec2(0.0, 6.0));
    }

    ui.painter()
        .hline(bar.x_range(), bar.bottom(), Stroke::new(1.0, LINE));

    if let Some(index) = close_at {
        shell.close_tab(index);
    }
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
    let sum: f32 = natural.iter().sum::<f32>() + 4.0 * natural.len().saturating_sub(1) as f32;
    if sum <= available {
        return natural;
    }
    let scale = available / sum;
    natural
        .into_iter()
        .map(|width| (width * scale).max(88.0))
        .collect()
}

fn natural_width(ui: &egui::Ui, tab: &Tab) -> f32 {
    let text = theme::measure(ui, tab.title(), 13.0);
    if tab.is_home() {
        (10.0 + 18.0 + 8.0 + text + 14.0).max(92.0)
    } else {
        (10.0 + 18.0 + 8.0 + text + 8.0 + 16.0 + 12.0).clamp(150.0, 210.0)
    }
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
    if selected {
        painter.rect_filled(
            rect,
            CornerRadius {
                nw: 8,
                ne: 8,
                sw: 0,
                se: 0,
            },
            WHITE,
        );
    } else if response.hovered() {
        painter.rect_filled(
            rect,
            CornerRadius {
                nw: 8,
                ne: 8,
                sw: 0,
                se: 0,
            },
            Color32::from_white_alpha(180),
        );
    }

    let badge_center = pos2(rect.left() + 10.0 + 9.0, rect.center().y);
    match tab.kind() {
        Some(kind) => theme::paint_badge(
            painter,
            badge_center,
            18.0,
            4,
            kind.color(),
            kind.badge(),
            11.0,
        ),
        None => {
            theme::paint_badge(painter, badge_center, 18.0, 4, theme::NEUTRAL, "", 11.0);
            let icon = Rect::from_center_size(badge_center, Vec2::splat(12.0));
            theme::paint_nav_icon(painter, icon, theme::NavIcon::Home, WHITE);
        }
    }

    let text_left = rect.left() + 10.0 + 18.0 + 8.0;
    let text_right = if tab.is_home() {
        rect.right() - 10.0
    } else {
        rect.right() - 28.0
    };
    let title = fit_title(ui, tab.title(), (text_right - text_left).max(8.0));
    let color = if selected {
        TEXT
    } else {
        Color32::from_rgb(0x44, 0x44, 0x44)
    };
    painter.text(
        pos2(text_left, rect.center().y),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(13.0),
        color,
    );

    let mut hit = TabHit::None;
    if !tab.is_home() {
        let close_rect =
            Rect::from_min_max(pos2(rect.right() - 24.0, rect.top()), rect.right_bottom());
        let close = ui.interact(
            close_rect,
            Id::new(("tab-close", tab.widget_id())),
            Sense::click(),
        );
        let close_color = if close.hovered() { TEXT } else { SUB };
        painter.text(
            close_rect.center(),
            Align2::CENTER_CENTER,
            "×",
            FontId::proportional(14.0),
            close_color,
        );
        if close.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if close.clicked() {
            hit = TabHit::Close;
        }
    }
    if matches!(hit, TabHit::None) && response.clicked() {
        hit = TabHit::Select;
    }
    hit
}

fn fit_title(ui: &egui::Ui, title: &str, max_width: f32) -> String {
    if theme::measure(ui, title, 13.0) <= max_width {
        return title.to_owned();
    }
    let mut owned: String = title.chars().collect();
    while !owned.is_empty() && theme::measure(ui, &format!("{owned}…"), 13.0) > max_width {
        owned.pop();
    }
    format!("{owned}…")
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
        let center = pos2(bar.left() + 18.0 + index as f32 * 20.0, cy);
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
