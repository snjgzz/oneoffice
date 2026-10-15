//! 文档标签里的占位界面：菜单、功能区、画布、状态栏。
//! 真正的 wordcraft / gridcraft / deckcraft 接进来之后，整块由它们的 `ui()` 绘制。

use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Pos2, Rect, Sense, Stroke, Vec2, pos2, vec2,
};

use crate::editor::DocumentEditor;
use crate::kind::DocKind;
use crate::theme::{
    self, BLUE, CALLOUT_BG, CALLOUT_FG, CALLOUT_LINE, CANVAS, LINE, SUB, TEXT, WHITE,
};

pub(crate) struct PlaceholderEditor {
    id: u64,
    kind: DocKind,
    title: String,
    menu: usize,
}

impl PlaceholderEditor {
    pub(crate) fn new(id: u64, kind: DocKind, title: String) -> Self {
        Self {
            id,
            kind,
            title,
            menu: 1,
        }
    }
}

impl DocumentEditor for PlaceholderEditor {
    fn kind(&self) -> DocKind {
        self.kind
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let menu = Rect::from_min_size(rect.min, vec2(rect.width(), 34.0));
        let ribbon = Rect::from_min_size(menu.left_bottom(), vec2(rect.width(), 92.0));
        let status =
            Rect::from_min_max(pos2(rect.left(), rect.bottom() - 26.0), rect.right_bottom());
        let canvas = Rect::from_min_max(ribbon.left_bottom(), pos2(rect.right(), status.top()));

        let painter = ui.painter();
        painter.rect_filled(menu, CornerRadius::ZERO, WHITE);
        painter.rect_filled(ribbon, CornerRadius::ZERO, WHITE);
        painter.rect_filled(canvas, CornerRadius::ZERO, CANVAS);
        painter.rect_filled(status, CornerRadius::ZERO, WHITE);
        let line = Stroke::new(1.0, LINE);
        painter.hline(menu.x_range(), menu.bottom(), line);
        painter.hline(ribbon.x_range(), ribbon.bottom(), line);
        painter.hline(status.x_range(), status.top(), line);

        self.paint_menu(ui, menu);
        paint_ribbon(ui, ribbon, self.kind);
        paint_canvas(ui, canvas, self.kind, &self.title);
        paint_status(ui, status, self.kind);
    }
}

impl PlaceholderEditor {
    fn paint_menu(&mut self, ui: &mut egui::Ui, menu: Rect) {
        let labels = self.kind.menus();
        let mut x = menu.left() + 18.0;
        for (index, label) in labels.iter().enumerate() {
            let width = theme::measure(ui, label, 13.0);
            let item = Rect::from_min_size(pos2(x, menu.top()), vec2(width, menu.height()));
            let response = ui.interact(item, Id::new(("doc-menu", self.id, index)), Sense::click());
            if response.clicked() {
                self.menu = index;
            }
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let selected = self.menu == index;
            let color = if selected { BLUE } else { TEXT };
            ui.painter().text(
                item.center(),
                Align2::CENTER_CENTER,
                *label,
                FontId::proportional(13.0),
                color,
            );
            if selected {
                ui.painter()
                    .hline(item.x_range(), item.bottom() - 1.0, Stroke::new(2.0, BLUE));
            }
            x += width + 22.0;
        }
    }
}

fn paint_ribbon(ui: &egui::Ui, ribbon: Rect, kind: DocKind) {
    let painter = ui.painter();
    let mut x = ribbon.left() + 16.0;
    let top = ribbon.top() + 12.0;
    for (label, widths) in kind.ribbon() {
        let group_w: f32 =
            widths.iter().sum::<f32>() + 8.0 * (widths.len().saturating_sub(1) as f32) + 16.0;
        let mut chip_x = x;
        for width in *widths {
            let chip = Rect::from_min_size(pos2(chip_x, top), vec2(*width, 26.0));
            painter.rect_filled(
                chip,
                CornerRadius::same(5),
                Color32::from_rgb(0xF0, 0xF2, 0xF5),
            );
            chip_x += width + 8.0;
        }
        painter.text(
            pos2(x + (group_w - 16.0) / 2.0, ribbon.bottom() - 16.0),
            Align2::CENTER_CENTER,
            *label,
            FontId::proportional(11.0),
            SUB,
        );
        let edge = x + group_w;
        painter.vline(edge, ribbon.y_range().shrink(14.0), Stroke::new(1.0, LINE));
        x = edge + 12.0;
        if x > ribbon.right() - 40.0 {
            break;
        }
    }
}

fn paint_canvas(ui: &egui::Ui, canvas: Rect, kind: DocKind, title: &str) {
    let painter = ui.painter();
    let page_w = (canvas.width() - 80.0).clamp(280.0, 640.0);
    let page_h = (canvas.height() - 48.0).clamp(180.0, 820.0);
    let page = Rect::from_min_size(
        pos2(canvas.center().x - page_w * 0.5, canvas.top() + 28.0),
        vec2(page_w, page_h),
    );
    painter.rect_filled(
        page.translate(vec2(0.0, 2.0)),
        CornerRadius::same(2),
        Color32::from_black_alpha(18),
    );
    painter.rect_filled(page, CornerRadius::ZERO, WHITE);

    let stem = title
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(title);
    painter.text(
        pos2(page.left() + 64.0, page.top() + 56.0),
        Align2::LEFT_TOP,
        stem,
        FontId::proportional(22.0),
        Color32::from_rgb(0x33, 0x33, 0x33),
    );
    let bar_colors = Color32::from_rgb(0xEE, 0xEE, 0xEE);
    let mut y = page.top() + 104.0;
    for factor in [0.90, 1.0, 0.70] {
        let bar = Rect::from_min_size(
            pos2(page.left() + 64.0, y),
            vec2((page_w - 128.0).max(40.0) * factor, 10.0),
        );
        painter.rect_filled(bar, CornerRadius::same(2), bar_colors);
        y += 22.0;
    }

    let note = format!(
        "此区域由嵌入的 {}（egui）渲染：功能区 + 画布",
        kind.embedded_app()
    );
    let galley = ui
        .painter()
        .layout(note, FontId::proportional(12.0), CALLOUT_FG, 460.0);
    let pad = Vec2::new(10.0, 6.0);
    let size = galley.size() + pad * 2.0;
    let callout = Rect::from_min_size(
        pos2(
            (canvas.right() - size.x - 20.0).max(canvas.left() + 12.0),
            canvas.top() + 10.0,
        ),
        size,
    );
    painter.rect_filled(callout, CornerRadius::same(6), CALLOUT_BG);
    dash_rect(painter, callout, CALLOUT_LINE);
    painter.galley(callout.min + pad, galley, CALLOUT_FG);
}

fn paint_status(ui: &egui::Ui, status: Rect, kind: DocKind) {
    let painter = ui.painter();
    let mut x = status.left() + 16.0;
    for label in kind.status_left() {
        painter.text(
            pos2(x, status.center().y),
            Align2::LEFT_CENTER,
            *label,
            FontId::proportional(12.0),
            SUB,
        );
        x += theme::measure(ui, label, 12.0) + 20.0;
    }
    painter.text(
        pos2(status.right() - 16.0, status.center().y),
        Align2::RIGHT_CENTER,
        "100%",
        FontId::proportional(12.0),
        SUB,
    );
}

fn dash_rect(painter: &egui::Painter, rect: Rect, color: Color32) {
    dash(painter, rect.left_top(), rect.right_top(), color);
    dash(painter, rect.right_top(), rect.right_bottom(), color);
    dash(painter, rect.right_bottom(), rect.left_bottom(), color);
    dash(painter, rect.left_bottom(), rect.left_top(), color);
}

fn dash(painter: &egui::Painter, a: Pos2, b: Pos2, color: Color32) {
    let delta = b - a;
    let len = delta.length();
    if len < 1.0 {
        return;
    }
    let step = delta / len;
    let mut cursor = 0.0;
    let mut on = true;
    while cursor < len {
        let next = (cursor + if on { 4.0 } else { 3.0 }).min(len);
        if on {
            painter.line_segment(
                [a + step * cursor, a + step * next],
                Stroke::new(1.0, color),
            );
        }
        cursor = next;
        on = !on;
    }
}
