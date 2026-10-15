//! 首页：侧栏、搜索、新建卡片、最近文件、设置占位。

use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, Rect, RichText, Sense,
    Stroke, TextEdit, TextWrapMode, Vec2, pos2, vec2,
};
use egui_extras::{Column, TableBuilder};

use crate::app::{HomeView, Shell};
use crate::kind::{self, DocKind};
use crate::platform::{self, reveal};
use crate::recent::RecentFile;
use crate::theme::{
    self, BLUE, HEADER_BG, HOVER, LINE, NAV_ON, NAV_TEXT, ROW_LINE, SUB, TEXT, WHITE,
};

impl Shell {
    pub(crate) fn show_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 2.0;
        brand(ui);
        ui.add_space(14.0);
        let new_button = new_button(ui);
        if new_button.clicked() {
            self.toggle_menu(new_button.rect.left_bottom() + vec2(0.0, 6.0));
        }
        ui.add_space(12.0);

        if nav(
            ui,
            "首页",
            theme::NavIcon::Home,
            self.home_view == HomeView::Home,
        )
        .clicked()
        {
            self.home_view = HomeView::Home;
        }
        if nav(
            ui,
            "最近",
            theme::NavIcon::Recent,
            self.home_view == HomeView::Recent,
        )
        .clicked()
        {
            self.home_view = HomeView::Recent;
        }

        let footer = 46.0;
        let gap = (ui.available_height() - footer).max(8.0);
        ui.add_space(gap);
        let width = ui.available_width();
        let (line, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
        ui.painter().hline(
            line.x_range().shrink(4.0),
            line.center().y,
            Stroke::new(1.0, LINE),
        );
        ui.add_space(8.0);
        if nav(
            ui,
            "设置",
            theme::NavIcon::Settings,
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
        self.top_bar(ui);
        ui.add_space(20.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.home_view == HomeView::Home {
                    section_title(ui, "新建");
                    ui.add_space(12.0);
                    self.new_cards(ui);
                    ui.add_space(28.0);
                }
                section_title(ui, "最近文件");
                ui.add_space(12.0);
                self.recent_table(ui);
                ui.add_space(8.0);
            });
    }

    fn show_settings(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("设置").size(15.0).color(TEXT));
        ui.add_space(12.0);
        ui.label(RichText::new("偏好设置还没有接入。这一页先占着位置。").color(TEXT));
        ui.add_space(18.0);
        ui.label(RichText::new("最近文件").size(13.0).color(SUB));
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
                .color(SUB),
        );
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            search_box(ui, &mut self.search, self.focus_search);
            self.focus_search = false;
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                avatar(ui);
            });
        });
    }

    fn new_cards(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            let gaps = 16.0 * 3.0;
            let width = ((ui.available_width() - gaps) / 4.0).max(120.0);
            for kind in [DocKind::Word, DocKind::Sheet, DocKind::Deck] {
                if card(
                    ui,
                    width,
                    kind.color(),
                    kind.badge(),
                    kind.new_label(),
                    kind.card_subtitle(),
                )
                .clicked()
                {
                    self.new_untitled(kind);
                }
            }
            let open_label = format!("从本地打开 · {}", platform::shortcut("O"));
            if card(
                ui,
                width,
                kind::open_card_color(),
                "",
                "打开文件",
                &open_label,
            )
            .clicked()
            {
                self.open_dialog();
            }
        });
    }

    fn recent_table(&mut self, ui: &mut egui::Ui) {
        let files = self.filtered_recent();
        let mut open_path = None;
        let mut remove_path = None;
        let mut reveal_path = None;

        Frame::new()
            .fill(WHITE)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(0))
            .show(ui, |ui| {
                if files.is_empty() {
                    empty_recent(ui, self.search.trim().is_empty());
                    return;
                }
                let width = ui.available_width();
                let cols = columns(width);
                let len = files.len();
                TableBuilder::new(ui)
                    .striped(false)
                    .vscroll(false)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(Column::exact(cols[0]))
                    .column(Column::exact(cols[1]))
                    .column(Column::exact(cols[2]))
                    .column(Column::remainder())
                    .sense(Sense::click())
                    .header(38.0, |mut header| {
                        for (index, title) in
                            ["名称", "类型", "位置", "修改时间"].into_iter().enumerate()
                        {
                            header.col(|ui| {
                                paint_cell_bg(ui, HEADER_BG);
                                if index == 0 {
                                    ui.add_space(18.0);
                                }
                                ui.label(RichText::new(title).size(12.0).color(SUB));
                            });
                        }
                    })
                    .body(|mut body| {
                        for (index, file) in files.iter().enumerate() {
                            let path = file.path.clone();
                            body.row(46.0, |mut row| {
                                row.col(|ui| {
                                    paint_cell_bg(
                                        ui,
                                        if ui.rect_contains_pointer(ui.max_rect()) {
                                            HOVER
                                        } else {
                                            WHITE
                                        },
                                    );
                                    ui.add_space(18.0);
                                    let (badge, _) =
                                        ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                                    theme::paint_badge(
                                        ui.painter(),
                                        badge.center(),
                                        18.0,
                                        4,
                                        file.kind.color(),
                                        file.kind.badge(),
                                        11.0,
                                    );
                                    ui.add_space(8.0);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(file.name()).size(13.0).color(TEXT),
                                        )
                                        .wrap_mode(TextWrapMode::Truncate),
                                    );
                                    cell_line(ui, index + 1 != len);
                                });
                                row.col(|ui| {
                                    paint_cell_bg(
                                        ui,
                                        if ui.rect_contains_pointer(ui.max_rect()) {
                                            HOVER
                                        } else {
                                            WHITE
                                        },
                                    );
                                    ui.label(
                                        RichText::new(file.kind.label()).size(13.0).color(SUB),
                                    );
                                    cell_line(ui, index + 1 != len);
                                });
                                row.col(|ui| {
                                    paint_cell_bg(
                                        ui,
                                        if ui.rect_contains_pointer(ui.max_rect()) {
                                            HOVER
                                        } else {
                                            WHITE
                                        },
                                    );
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(file.location()).size(13.0).color(SUB),
                                        )
                                        .wrap_mode(TextWrapMode::Truncate),
                                    );
                                    cell_line(ui, index + 1 != len);
                                });
                                row.col(|ui| {
                                    paint_cell_bg(
                                        ui,
                                        if ui.rect_contains_pointer(ui.max_rect()) {
                                            HOVER
                                        } else {
                                            WHITE
                                        },
                                    );
                                    ui.label(
                                        RichText::new(file.modified_label()).size(13.0).color(SUB),
                                    );
                                    cell_line(ui, index + 1 != len);
                                });
                                let response = row.response();
                                if response.double_clicked() {
                                    open_path = Some(path.clone());
                                }
                                response.context_menu(|ui| {
                                    if ui.button("打开所在文件夹").clicked() {
                                        reveal_path = Some(path.clone());
                                    }
                                    if ui.button("移除记录").clicked() {
                                        remove_path = Some(path.clone());
                                    }
                                });
                            });
                        }
                    });
            });

        if let Some(path) = reveal_path {
            reveal(&path);
        }
        if let Some(path) = remove_path {
            self.remove_recent(&path);
        }
        if let Some(path) = open_path {
            self.open_existing(path);
        }
    }

    fn filtered_recent(&self) -> Vec<RecentFile> {
        let query = self.search.trim().to_lowercase();
        self.recent
            .files
            .iter()
            .filter(|file| query.is_empty() || file.name().to_lowercase().contains(&query))
            .cloned()
            .collect()
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
                    .stroke(Stroke::new(1.0, LINE))
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

        let pressed_outside = ctx.input(|input| input.pointer.any_pressed())
            && !area.response.hovered()
            && !self.menu_just_opened;
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

enum MenuAction {
    New(DocKind),
    Open,
}

fn brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add_space(4.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
        theme::paint_badge(ui.painter(), rect.center(), 26.0, 6, BLUE, "办", 14.0);
        ui.label(RichText::new("OneOffice").size(17.0).color(TEXT));
    });
}

fn new_button(ui: &mut egui::Ui) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 38.0), Sense::click());
    let fill = if response.hovered() {
        Color32::from_rgb(0x2A, 0x64, 0xD6)
    } else {
        BLUE
    };
    ui.painter().rect_filled(rect, CornerRadius::same(8), fill);
    let center = rect.center();
    theme::paint_plus(ui.painter(), center + vec2(-28.0, 0.0), 5.0, 1.6, WHITE);
    ui.painter().text(
        center + vec2(8.0, 0.0),
        Align2::CENTER_CENTER,
        "新建",
        FontId::proportional(14.0),
        WHITE,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn nav(ui: &mut egui::Ui, label: &str, icon: theme::NavIcon, selected: bool) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    if selected {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(8), NAV_ON);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(8), HOVER);
    }
    let color = if selected { BLUE } else { NAV_TEXT };
    let icon_rect = Rect::from_min_size(
        pos2(rect.left() + 12.0, rect.center().y - 9.0),
        Vec2::splat(18.0),
    );
    theme::paint_nav_icon(ui.painter(), icon_rect, icon, color);
    ui.painter().text(
        pos2(rect.left() + 40.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        color,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(15.0).color(TEXT));
}

fn search_box(ui: &mut egui::Ui, text: &mut String, focus: bool) {
    let width = ui.available_width().clamp(180.0, 520.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 38.0), Sense::hover());
    let stroke =
        if response.hovered() || ui.memory(|memory| memory.has_focus(Id::new("home-search"))) {
            Stroke::new(1.0, BLUE)
        } else {
            Stroke::new(1.0, LINE)
        };
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(19),
        stroke,
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .rect_filled(rect.shrink(1.0), CornerRadius::same(18), WHITE);
    theme::paint_search(ui.painter(), pos2(rect.left() + 20.0, rect.center().y), SUB);

    let text_rect = Rect::from_min_max(
        pos2(rect.left() + 36.0, rect.top()),
        pos2(rect.right() - 14.0, rect.bottom()),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let hint = format!("搜索文档（{}）", platform::shortcut("K"));
    let output = TextEdit::singleline(text)
        .id(Id::new("home-search"))
        .hint_text(hint)
        .frame(Frame::NONE)
        .desired_width(text_rect.width())
        .show(&mut child);
    if focus {
        output.response.request_focus();
    }
}

fn avatar(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
    ui.painter()
        .circle_filled(rect.center(), 16.0, Color32::from_rgb(0xC9, 0xD6, 0xF5));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "我",
        FontId::proportional(13.0),
        BLUE,
    );
}

fn card(
    ui: &mut egui::Ui,
    width: f32,
    color: Color32,
    letter: &str,
    title: &str,
    subtitle: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 118.0), Sense::click());
    let stroke = if response.hovered() {
        Stroke::new(1.0, BLUE)
    } else {
        Stroke::new(1.0, LINE)
    };
    let fill = if response.hovered() {
        Color32::from_rgb(0xF8, 0xFA, 0xFF)
    } else {
        WHITE
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(12),
        fill,
        stroke,
        egui::StrokeKind::Inside,
    );
    let badge = Rect::from_min_size(
        pos2(rect.left() + 18.0, rect.top() + 18.0),
        Vec2::splat(40.0),
    );
    theme::paint_badge(ui.painter(), badge.center(), 40.0, 10, color, letter, 18.0);
    if letter.is_empty() {
        theme::paint_arrow_ne(ui.painter(), badge.shrink(6.0), WHITE);
    }
    ui.painter().text(
        pos2(rect.left() + 18.0, rect.bottom() - 40.0),
        Align2::LEFT_TOP,
        title,
        FontId::proportional(15.0),
        TEXT,
    );
    ui.painter().text(
        pos2(rect.left() + 18.0, rect.bottom() - 16.0),
        Align2::LEFT_BOTTOM,
        subtitle,
        FontId::proportional(12.0),
        SUB,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn menu_row(ui: &mut egui::Ui, label: &str) -> bool {
    let width = ui.available_width().max(168.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(6), NAV_ON);
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
        ui.label(RichText::new(detail).size(12.0).color(SUB));
    });
}

fn columns(width: f32) -> [f32; 3] {
    let kind = 88.0;
    let time = 120.0;
    let rest = (width - kind - time).max(220.0);
    let name = rest * 0.56;
    let location = rest - name;
    [name, kind, location]
}

fn paint_cell_bg(ui: &egui::Ui, color: Color32) {
    ui.painter()
        .rect_filled(ui.max_rect(), CornerRadius::ZERO, color);
}

fn cell_line(ui: &egui::Ui, draw: bool) {
    if draw {
        let rect = ui.max_rect();
        ui.painter()
            .hline(rect.x_range(), rect.bottom(), Stroke::new(1.0, ROW_LINE));
    }
}
