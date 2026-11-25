use adw::prelude::*;
use ebook_parsers::ParsedBook;
use gtk4::{
    glib, Box, GestureClick, Label, Orientation, PolicyType, Revealer,
    RevealerTransitionType, ScrolledWindow, Separator, Stack, ToggleButton, Widget,
};
use std::rc::Rc;
use std::time::Duration;

pub const SIDEBAR_TRANSITION_MS: u64 = 220;
const READER_SIDEBAR_WIDTH: i32 = 195;

struct TocRow {
    row_box: Box,
    title_lbl: Label,
}

pub struct ReaderSidebar {
    pub widget: Widget,
    revealer: Revealer,
    highlight_chapter_fn: Rc<dyn Fn(usize)>,
}

impl ReaderSidebar {
    pub fn new(book: &ParsedBook, on_chapter_selected: Rc<dyn Fn(usize)>) -> Self {
        let sidebar = Box::new(Orientation::Vertical, 0);
        sidebar.set_size_request(READER_SIDEBAR_WIDTH, -1);
        sidebar.add_css_class("sidebar");

        let tabs_bar = Box::new(Orientation::Horizontal, 0);
        tabs_bar.set_homogeneous(true);

        let tab_toc = ToggleButton::builder()
            .label("Оглавление")
            .active(true)
            .css_classes(vec!["reader-tab".to_string()])
            .build();

        let tab_notes = ToggleButton::builder()
            .label("Заметки")
            .group(&tab_toc)
            .css_classes(vec!["reader-tab".to_string()])
            .build();

        tabs_bar.append(&tab_toc);
        tabs_bar.append(&tab_notes);
        sidebar.append(&tabs_bar);
        sidebar.append(&Separator::new(Orientation::Horizontal));

        let stack = Stack::new();
        stack.set_vexpand(true);

        let (toc_widget, highlight_chapter_fn) = build_toc_tab(book, on_chapter_selected);
        stack.add_named(&toc_widget, Some("toc"));
        stack.add_named(&build_notes_tab(), Some("notes"));

        let stack_for_toc = stack.clone();
        tab_toc.connect_toggled(move |btn| {
            if btn.is_active() {
                stack_for_toc.set_visible_child_name("toc");
            }
        });

        let stack_for_notes = stack.clone();
        tab_notes.connect_toggled(move |btn| {
            if btn.is_active() {
                stack_for_notes.set_visible_child_name("notes");
            }
        });

        sidebar.append(&stack);

        let sidebar_bottom = Box::new(Orientation::Vertical, 4);
        sidebar_bottom.set_margin_start(12);
        sidebar_bottom.set_margin_end(12);
        sidebar_bottom.set_margin_top(8);
        sidebar_bottom.set_margin_bottom(14);

        let format_str = match book.format {
            ebook_core::BookFormat::Txt => "TXT",
            ebook_core::BookFormat::Fb2 => "FB2",
            ebook_core::BookFormat::Epub => "EPUB",
        };
        let meta_label = Label::builder()
            .label(&format!("{} · Глав: {}", format_str, book.chapters.len()))
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        sidebar_bottom.append(&meta_label);
        sidebar.append(&sidebar_bottom);

        let revealer = Revealer::builder()
            .transition_type(RevealerTransitionType::SlideRight)
            .transition_duration(SIDEBAR_TRANSITION_MS as u32)
            .child(&sidebar)
            .reveal_child(true)
            .build();

        let widget = revealer.clone().upcast();

        Self {
            widget,
            revealer,
            highlight_chapter_fn,
        }
    }

    pub fn toggle(&self) {
        let next_open = !self.revealer.reveals_child();
        if next_open {
            self.revealer.set_visible(true);
            self.revealer.set_reveal_child(true);
        } else {
            self.revealer.set_reveal_child(false);
            let rev_hide = self.revealer.clone();
            glib::timeout_add_local_once(
                Duration::from_millis(SIDEBAR_TRANSITION_MS),
                move || {
                    if !rev_hide.reveals_child() {
                        rev_hide.set_visible(false);
                    }
                },
            );
        }
    }

    pub fn highlight_chapter(&self, chapter_idx: usize) {
        (self.highlight_chapter_fn)(chapter_idx);
    }
}

fn build_toc_tab(
    book: &ParsedBook,
    on_chapter_selected: Rc<dyn Fn(usize)>,
) -> (Widget, Rc<dyn Fn(usize)>) {
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .build();

    let list = Box::new(Orientation::Vertical, 4);
    list.set_margin_start(8);
    list.set_margin_end(8);
    list.set_margin_top(10);
    list.set_margin_bottom(10);

    let mut rows = Vec::with_capacity(book.chapters.len());

    for (i, chapter) in book.chapters.iter().enumerate() {
        let row_box = Box::new(Orientation::Horizontal, 8);
        row_box.set_margin_start(8);
        row_box.set_margin_end(8);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.add_css_class("chapter-row-box");

        let title_lbl = Label::builder()
            .label(&chapter.title)
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .hexpand(true)
            .css_classes(vec!["chapter-title-text".to_string()])
            .build();

        row_box.append(&title_lbl);

        let click_controller = GestureClick::new();
        let cb_select = on_chapter_selected.clone();

        click_controller.connect_released(move |_, _, _, _| {
            cb_select(i);
        });

        row_box.add_controller(click_controller);
        list.append(&row_box);

        rows.push(TocRow { row_box, title_lbl });
    }

    scroll.set_child(Some(&list));

    let rows_rc = Rc::new(rows);
    let highlight_chapter: Rc<dyn Fn(usize)> = {
        let rows_ref = rows_rc.clone();
        Rc::new(move |active_idx: usize| {
            for (idx, item) in rows_ref.iter().enumerate() {
                if idx == active_idx {
                    item.row_box.add_css_class("active-chapter");
                    item.title_lbl.add_css_class("heading");
                } else {
                    item.row_box.remove_css_class("active-chapter");
                    item.title_lbl.remove_css_class("heading");
                }
            }
        })
    };

    (scroll.upcast(), highlight_chapter)
}

fn build_notes_tab() -> Widget {
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .build();

    let list = Box::new(Orientation::Vertical, 10);
    list.set_margin_start(12);
    list.set_margin_end(12);
    list.set_margin_top(14);
    list.set_margin_bottom(12);

    let empty_lbl = Label::builder()
        .label("Здесь пока нет заметок")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label".to_string(), "caption".to_string()])
        .build();
    list.append(&empty_lbl);

    scroll.set_child(Some(&list));
    scroll.upcast()
}
