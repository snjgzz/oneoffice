//! 首页：浅色侧栏、最近文件、设置占位。打开文档后侧栏不画，由壳的主布局决定。

use std::path::PathBuf;

use chrono::Local;
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, Rect, RichText, Sense,
    Stroke, StrokeKind, Vec2, pos2, vec2,
};

use crate::app::{HomeView, Shell};
use crate::kind::DocKind;
use crate::platform::reveal;
use crate::recent::{self, RecentFile, RecentSpan};
use crate::theme::{
    self, ACCENT_DEEP, ACCENT_SOFT, BORDER, BORDER_SOFT, CHIP_LINE, ELEV, HOVER, NAV_ON, TAG_BG,
    TAG_FG, TEXT, TEXT_2, TEXT_3, TEXT_4, WHITE,
};

const ICON_32: &[u8] = include_bytes!("../../../packaging/macos/OneOffice.iconset/icon_32x32.png");

impl Shell {
    pub(crate) fn show_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.add_space(2.0);
        brand(ui);
        ui.add_space(14.0);
        quick_cards(ui, self);
        ui.add_space(8.0);
        if open_button(ui).clicked() {
            self.open_dialog();
        }
        ui.add_space(14.0);
        if nav(
            ui,
            "最近",
            theme::Mark::Clock,
            self.home_view == HomeView::Home,
        )
        .clicked()
        {
            self.home_view = HomeView::Home;
        }

        let footer = 1.0 + 10.0 + 36.0;
        ui.add_space((ui.available_height() - footer).max(8.0));
        let width = ui.available_width();
        let (line, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
        ui.painter().hline(
            line.x_range().shrink(2.0),
            line.center().y,
            Stroke::new(1.0, BORDER_SOFT),
        );
        ui.add_space(10.0);
        if nav(
            ui,
            "设置",
            theme::Mark::Gear,
            self.home_view == HomeView::Settings,
        )
        .clicked()
        {
            self.home_view = HomeView::Settings;
        }
    }

    pub(crate) fn show_home_main(&mut self, ui: &mut egui::Ui) {
        if self.home_view == HomeView::Settings {
            self.show_settings(ui);
            return;
        }
        self.recent_header(ui);
        ui.add_space(12.0);
        let files = self.filtered_recent();
        let grid = self.recent_grid;
        let no_files = self.recent.files.is_empty();
        let mut action = FileAction::default();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                if files.is_empty() {
                    if !grid {
                        column_header(ui);
                    }
                    empty_recent(ui, no_files);
                    return;
                }
                if grid {
                    paint_grid(ui, &files, &mut action);
                } else {
                    column_header(ui);
                    paint_groups(ui, &files, &mut action);
                }
                ui.add_space(12.0);
            });
        apply_file_action(self, action);
    }

    fn show_settings(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("设置").size(18.0).color(TEXT));
        ui.add_space(8.0);
        ui.label(RichText::new("偏好设置还没有接入。这一页先占着位置。").color(TEXT_2));
        ui.add_space(18.0);
        ui.label(RichText::new("最近文件").size(13.0).color(TEXT_3));
        ui.add_space(4.0);
        ui.label(
            RichText::new(self.recent_path.display().to_string())
                .size(13.0)
                .color(TEXT),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("当前 {} 条", self.recent.files.len()))
                .size(12.0)
                .color(TEXT_4),
        );
    }

    fn recent_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.label(RichText::new("最近").size(18.0).color(TEXT));
            if icon_button(ui, theme::Mark::Refresh, false).clicked() {
                self.reload_recent();
            }
        });
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let filters = [
                ("全部", None),
                ("文字", Some(DocKind::Word)),
                ("表格", Some(DocKind::Sheet)),
                ("演示", Some(DocKind::Deck)),
            ];
            for (label, kind) in filters {
                if chip(ui, label, self.recent_kind == kind).clicked() {
                    self.recent_kind = kind;
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if icon_button(ui, theme::Mark::Grid, self.recent_grid).clicked() {
                    self.recent_grid = true;
                }
                if icon_button(ui, theme::Mark::List, !self.recent_grid).clicked() {
                    self.recent_grid = false;
                }
            });
        });
    }

    fn filtered_recent(&self) -> Vec<RecentFile> {
        let query = self.search.trim().to_lowercase();
        let mut files: Vec<RecentFile> = self
            .recent
            .files
            .iter()
            .filter(|file| {
                self.recent_kind.is_none_or(|kind| kind == file.kind)
                    && (query.is_empty()
                        || file.name().to_lowercase().contains(&query)
                        || file.location().to_lowercase().contains(&query))
            })
            .cloned()
            .collect();
        files.sort_by_key(|file| std::cmp::Reverse(file.stamp_ms()));
        files
    }

    pub(crate) fn show_new_menu(&mut self, ctx: &egui::Context) {
        let Some(pos) = self.menu else {
            self.menu_just_opened = false;
            return;
        };
        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.menu = None;
            self.menu_just_opened = false;
            return;
        }

        let mut action = None;
        let area = egui::Area::new(Id::new("new-doc-menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .constrain(true)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(WHITE)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(6))
                    .shadow(ui.visuals().window_shadow)
                    .show(ui, |ui| {
                        ui.set_min_width(180.0);
                        ui.spacing_mut().item_spacing.y = 2.0;
                        if menu_row(ui, "新建文字") {
                            action = Some(MenuAction::New(DocKind::Word));
                        }
                        if menu_row(ui, "新建表格") {
                            action = Some(MenuAction::New(DocKind::Sheet));
                        }
                        if menu_row(ui, "新建演示") {
                            action = Some(MenuAction::New(DocKind::Deck));
                        }
                        ui.separator();
                        if menu_row(ui, "打开文件") {
                            action = Some(MenuAction::Open);
                        }
                    });
            });

        // 菜单行盖在 Area 上，行被悬停时 Area 自己的 hovered() 是 false。
        // 用它判断「点到外面」会在按下那一帧关掉菜单，松开时点不到行，新建不会发生。
        let pointer_inside = ctx.input(|input| {
            input
                .pointer
                .interact_pos()
                .is_some_and(|pos| area.response.rect.contains(pos))
        });
        let pressed_outside = menu_press_is_outside(
            self.menu_just_opened,
            ctx.input(|input| input.pointer.any_pressed()),
            pointer_inside,
        );
        if action.is_some() || pressed_outside {
            self.menu = None;
        }
        self.menu_just_opened = false;

        match action {
            Some(MenuAction::New(kind)) => self.new_untitled(kind),
            Some(MenuAction::Open) => self.open_dialog(),
            None => {}
        }
    }
}

#[derive(Default)]
struct FileAction {
    open: Option<PathBuf>,
    remove: Option<PathBuf>,
    reveal: Option<PathBuf>,
}

enum MenuAction {
    New(DocKind),
    Open,
}

fn apply_file_action(shell: &mut Shell, action: FileAction) {
    if let Some(path) = action.reveal {
        reveal(&path);
    }
    if let Some(path) = action.remove {
        shell.remove_recent(&path);
    }
    if let Some(path) = action.open {
        shell.open_existing(path);
    }
}

fn brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.add_space(8.0);
        let cursor = ui.cursor().min;
        let shadow = Rect::from_min_size(cursor, Vec2::splat(32.0)).translate(vec2(0.0, 2.0));
        ui.painter().rect_filled(
            shadow,
            CornerRadius::same(8),
            theme::tint(theme::ACCENT_2, 46),
        );
        ui.add(
            egui::Image::from_bytes("bytes://oneoffice-icon-32.png", ICON_32)
                .corner_radius(CornerRadius::same(8))
                .fit_to_exact_size(vec2(32.0, 32.0)),
        );
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(RichText::new("OneOffice").size(15.0).strong().color(TEXT));
            ui.label(RichText::new("开源办公套件").size(11.0).color(TEXT_4));
        });
    });
}

fn quick_cards(ui: &mut egui::Ui, shell: &mut Shell) {
    let gap = 6.0;
    let width = (ui.available_width() - 4.0).max(0.0);
    let card_w = ((width - gap * 2.0) / 3.0).max(36.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        ui.add_space(2.0);
        for kind in [DocKind::Word, DocKind::Sheet, DocKind::Deck] {
            if qcard(ui, card_w, kind).clicked() {
                shell.new_untitled(kind);
            }
        }
    });
}

fn qcard(ui: &mut egui::Ui, width: f32, kind: DocKind) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 62.0), Sense::click());
    let line = if response.hovered() {
        ACCENT_DEEP
    } else {
        BORDER
    };
    let fill = if response.hovered() { HOVER } else { WHITE };
    ui.painter().rect(
        rect,
        CornerRadius::same(10),
        fill,
        Stroke::new(1.0, line),
        StrokeKind::Inside,
    );
    theme::paint_type_icon(
        ui.painter(),
        pos2(rect.center().x, rect.top() + 22.0),
        &theme::MarkSpec {
            badge: 26.0,
            radius: 7,
            fill: theme::tint(kind.color(), 31),
            ink: kind.color(),
            mark: kind.mark(),
            glyph: 14.0,
        },
    );
    ui.painter().text(
        pos2(rect.center().x, rect.bottom() - 12.0),
        Align2::CENTER_CENTER,
        kind.label(),
        FontId::proportional(11.5),
        TEXT_2,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn open_button(ui: &mut egui::Ui) -> egui::Response {
    let width = (ui.available_width() - 4.0).max(0.0);
    ui.add_space(2.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    let fill = if response.hovered() { HOVER } else { WHITE };
    ui.painter().rect(
        rect,
        CornerRadius::same(9),
        fill,
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );
    let label = "打开…";
    let text_w = theme::measure(ui, label, 13.0);
    let total = 15.0 + 8.0 + text_w;
    let start = rect.center().x - total / 2.0;
    let icon = Rect::from_center_size(pos2(start + 7.5, rect.center().y), Vec2::splat(15.0));
    theme::paint_mark(ui.painter(), icon, theme::Mark::Folder, TEXT_2);
    ui.painter().text(
        pos2(start + 23.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        TEXT_2,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn nav(ui: &mut egui::Ui, label: &str, icon: theme::Mark, selected: bool) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    if selected {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(8), NAV_ON);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(8), HOVER);
    }
    let color = if selected { ACCENT_DEEP } else { TEXT_2 };
    let icon_color = if selected { ACCENT_DEEP } else { TEXT_3 };
    let icon_rect =
        Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), Vec2::splat(16.0));
    theme::paint_mark(ui.painter(), icon_rect, icon, icon_color);
    ui.painter().text(
        pos2(rect.left() + 38.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        color,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn chip(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let width = theme::measure(ui, label, 12.5) + 24.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, 28.0), Sense::click());
    if selected {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(14), ACCENT_SOFT);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(14),
            Stroke::new(1.0, CHIP_LINE),
            StrokeKind::Inside,
        );
    } else {
        let fill = if response.hovered() {
            HOVER
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, CornerRadius::same(14), fill);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(14),
            Stroke::new(1.0, BORDER),
            StrokeKind::Inside,
        );
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.5),
        if selected { ACCENT_DEEP } else { TEXT_3 },
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn icon_button(ui: &mut egui::Ui, mark: theme::Mark, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, CornerRadius::same(7), ELEV);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(7), HOVER);
    }
    let color = if selected { TEXT_2 } else { TEXT_4 };
    let icon = Rect::from_center_size(rect.center(), Vec2::splat(14.0));
    theme::paint_mark(ui.painter(), icon, mark, color);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn column_header(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
    let [name, location, _] = column_widths(rect.width());
    let y = rect.center().y;
    let labels = [
        (rect.left() + 12.0, "名称"),
        (rect.left() + 12.0 + name, "位置"),
        (rect.left() + 12.0 + name + location, "修改时间"),
    ];
    for (x, label) in labels {
        ui.painter().text(
            pos2(x, y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.0),
            TEXT_4,
        );
    }
    ui.painter()
        .hline(rect.x_range(), rect.bottom(), Stroke::new(1.0, BORDER_SOFT));
}

fn column_widths(total: f32) -> [f32; 3] {
    let inner = (total - 24.0).max(0.0);
    let date = 130.0_f32.min(inner * 0.30);
    let location = (inner - date) * 0.36;
    let name = inner - date - location;
    [name, location, date]
}

fn paint_groups(ui: &mut egui::Ui, files: &[RecentFile], action: &mut FileAction) {
    let now = Local::now().naive_local();
    for span in [RecentSpan::Week, RecentSpan::Month, RecentSpan::Earlier] {
        let group: Vec<&RecentFile> = files
            .iter()
            .filter(|file| recent::span_of_ms(file.stamp_ms(), now) == span)
            .collect();
        if group.is_empty() {
            continue;
        }
        group_label(ui, span.label());
        for file in group {
            file_row(ui, file, action);
        }
    }
}

fn paint_grid(ui: &mut egui::Ui, files: &[RecentFile], action: &mut FileAction) {
    let now = Local::now().naive_local();
    for span in [RecentSpan::Week, RecentSpan::Month, RecentSpan::Earlier] {
        let group: Vec<&RecentFile> = files
            .iter()
            .filter(|file| recent::span_of_ms(file.stamp_ms(), now) == span)
            .collect();
        if group.is_empty() {
            continue;
        }
        group_label(ui, span.label());
        let avail = ui.available_width();
        let gap = 10.0;
        let cols = ((avail + gap) / 230.0).floor().max(1.0);
        let card_w = (avail - gap * (cols - 1.0)) / cols;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, gap);
            for file in group {
                grid_card(ui, card_w, file, action);
            }
        });
        ui.add_space(8.0);
    }
}

fn group_label(ui: &mut egui::Ui, text: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
    let text_w = theme::measure(ui, text, 12.0);
    ui.painter().text(
        pos2(rect.left() + 12.0, rect.center().y + 4.0),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(12.0),
        TEXT_4,
    );
    let line_y = rect.center().y + 4.0;
    ui.painter().hline(
        (rect.left() + 12.0 + text_w + 8.0)..=(rect.right() - 12.0),
        line_y,
        Stroke::new(1.0, BORDER_SOFT),
    );
}

fn file_row(ui: &mut egui::Ui, file: &RecentFile, action: &mut FileAction) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(8), HOVER);
    }
    let [name_w, location_w, _] = column_widths(rect.width());
    theme::paint_type_icon(
        ui.painter(),
        pos2(rect.left() + 12.0 + 15.0, rect.center().y),
        &theme::MarkSpec {
            badge: 30.0,
            radius: 7,
            fill: theme::tint(file.kind.color(), 26),
            ink: file.kind.color(),
            mark: file.kind.mark(),
            glyph: 15.0,
        },
    );
    let name_left = rect.left() + 12.0 + 30.0 + 12.0;
    let name_right = rect.left() + 12.0 + name_w;
    paint_name_tag(ui, file, name_left, name_right, rect.center().y);
    let location = theme::fit_text(ui, &file.location(), 12.5, (location_w - 8.0).max(8.0));
    ui.painter().text(
        pos2(rect.left() + 12.0 + name_w, rect.center().y),
        Align2::LEFT_CENTER,
        location,
        FontId::proportional(12.5),
        TEXT_3,
    );
    ui.painter().text(
        pos2(rect.left() + 12.0 + name_w + location_w, rect.center().y),
        Align2::LEFT_CENTER,
        file.modified_label(),
        FontId::proportional(12.5),
        TEXT_3,
    );
    note_file_response(&response, file, action);
}

fn grid_card(ui: &mut egui::Ui, width: f32, file: &RecentFile, action: &mut FileAction) {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 76.0), Sense::click());
    let fill = if response.hovered() { HOVER } else { WHITE };
    ui.painter().rect(
        rect,
        CornerRadius::same(10),
        fill,
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );
    theme::paint_type_icon(
        ui.painter(),
        pos2(rect.left() + 16.0 + 15.0, rect.center().y),
        &theme::MarkSpec {
            badge: 30.0,
            radius: 7,
            fill: theme::tint(file.kind.color(), 26),
            ink: file.kind.color(),
            mark: file.kind.mark(),
            glyph: 15.0,
        },
    );
    let text_left = rect.left() + 16.0 + 30.0 + 10.0;
    let text_right = rect.right() - 12.0;
    paint_name_tag(ui, file, text_left, text_right, rect.center().y - 10.0);
    let meta = format!("{} · {}", file.location(), file.modified_label());
    let meta = theme::fit_text(ui, &meta, 12.0, (text_right - text_left).max(8.0));
    ui.painter().text(
        pos2(text_left, rect.center().y + 12.0),
        Align2::LEFT_CENTER,
        meta,
        FontId::proportional(12.0),
        TEXT_3,
    );
    note_file_response(&response, file, action);
}

fn paint_name_tag(ui: &egui::Ui, file: &RecentFile, left: f32, right: f32, y: f32) {
    let tag = "本地";
    let tag_w = theme::measure(ui, tag, 11.0) + 12.0;
    let title_max = (right - left - 8.0 - tag_w).max(8.0);
    let title = theme::fit_text(ui, &file.name(), 13.0, title_max);
    let title_w = theme::measure(ui, &title, 13.0).min(title_max);
    ui.painter().text(
        pos2(left, y),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(13.0),
        TEXT,
    );
    let tag_rect = Rect::from_center_size(
        pos2(left + title_w + 8.0 + tag_w / 2.0, y),
        vec2(tag_w, 16.0),
    );
    ui.painter()
        .rect_filled(tag_rect, CornerRadius::same(4), TAG_BG);
    ui.painter().rect_stroke(
        tag_rect,
        CornerRadius::same(4),
        Stroke::new(1.0, BORDER_SOFT),
        StrokeKind::Inside,
    );
    ui.painter().text(
        tag_rect.center(),
        Align2::CENTER_CENTER,
        tag,
        FontId::proportional(11.0),
        TAG_FG,
    );
}

fn note_file_response(response: &egui::Response, file: &RecentFile, action: &mut FileAction) {
    if response.hovered() {
        response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.double_clicked() {
        action.open = Some(file.path.clone());
    }
    let path = file.path.clone();
    response.context_menu(|ui| {
        if ui.button("打开所在文件夹").clicked() {
            action.reveal = Some(path.clone());
        }
        if ui.button("移除记录").clicked() {
            action.remove = Some(path.clone());
        }
    });
}

fn empty_recent(ui: &mut egui::Ui, no_files: bool) {
    ui.set_min_height(148.0);
    ui.vertical_centered(|ui| {
        ui.add_space(46.0);
        let (title, detail) = if no_files {
            (
                "还没有最近文件",
                "打开 .docx、.xlsx 或 .pptx 后会显示在这里",
            )
        } else {
            ("没有匹配的文件", "换个关键词，或清空搜索框")
        };
        ui.label(RichText::new(title).size(14.0).color(TEXT));
        ui.add_space(6.0);
        ui.label(RichText::new(detail).size(12.0).color(TEXT_4));
    });
}

fn menu_row(ui: &mut egui::Ui, label: &str) -> bool {
    let width = ui.available_width().max(168.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), HOVER);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    ui.painter().text(
        pos2(rect.left() + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        TEXT,
    );
    response.clicked()
}

/// 按下指针时要不要关掉新建菜单。刚打开的那一帧不算，菜单矩形里的按下也不算。
fn menu_press_is_outside(just_opened: bool, pressed: bool, pointer_inside_menu: bool) -> bool {
    pressed && !pointer_inside_menu && !just_opened
}

#[cfg(test)]
mod tests {
    use super::{column_widths, menu_press_is_outside};

    #[test]
    fn pressing_a_menu_row_does_not_count_as_outside() {
        assert!(!menu_press_is_outside(false, true, true));
        assert!(menu_press_is_outside(false, true, false));
        assert!(!menu_press_is_outside(true, true, false));
        assert!(!menu_press_is_outside(false, false, false));
    }

    #[test]
    fn columns_use_the_inner_width() {
        let total = 1000.0;
        let [name, location, date] = column_widths(total);
        let inner = total - 24.0;
        assert!((name + location + date - inner).abs() < 0.01);
        assert!(date <= 130.0);
        assert!(name > location);
    }
}
