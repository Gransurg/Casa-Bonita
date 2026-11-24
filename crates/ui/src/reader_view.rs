use adw::prelude::*;
use ebook_parsers::ParsedBook;
use gtk4::{Box, Button, Label, Orientation, PolicyType, ScrolledWindow, Widget};
use std::cell::RefCell;
use std::rc::Rc;

use crate::pagination::{estimate_remaining_minutes, paginate_chapter_content};
use crate::reader_nav::ReaderNavBar;
use crate::reader_sidebar::ReaderSidebar;

pub fn create_reader_view<F>(book: &ParsedBook, on_back_to_shelf: F) -> Widget
where
    F: Fn() + 'static + Clone,
{
    let root = Box::new(Orientation::Horizontal, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let current_chapter_idx = Rc::new(RefCell::new(0_usize));
    let current_page_idx = Rc::new(RefCell::new(0_usize));
    let total_chapters = book.chapters.len();

    let sidebar_slot: Rc<RefCell<Option<Rc<ReaderSidebar>>>> = Rc::new(RefCell::new(None));
    let nav_slot: Rc<RefCell<Option<Rc<ReaderNavBar>>>> = Rc::new(RefCell::new(None));

    // --- Центральная область текста ---
    let text_container = Box::new(Orientation::Vertical, 0);
    text_container.set_hexpand(true);
    text_container.set_vexpand(true);

    let top_bar = Box::new(Orientation::Horizontal, 12);
    top_bar.set_margin_start(20);
    top_bar.set_margin_end(20);
    top_bar.set_margin_top(14);
    top_bar.set_margin_bottom(8);

    let back_btn = Button::builder()
        .label("← На полку")
        .css_classes(vec!["suggested-action".to_string(), "pill".to_string()])
        .valign(gtk4::Align::Center)
        .build();

    let cb_back = on_back_to_shelf;
    back_btn.connect_clicked(move |_| {
        cb_back();
    });

    let toggle_sidebar_btn = Button::builder()
        .icon_name("sidebar-show-symbolic")
        .tooltip_text("Скрыть / открыть боковую панель")
        .css_classes(vec!["flat".to_string()])
        .valign(gtk4::Align::Center)
        .build();

    {
        let sb_slot = sidebar_slot.clone();
        toggle_sidebar_btn.connect_clicked(move |_| {
            if let Some(sb) = sb_slot.borrow().as_ref() {
                sb.toggle();
            }
        });
    }

    let page_header = Box::new(Orientation::Vertical, 2);
    page_header.set_halign(gtk4::Align::Center);
    page_header.set_hexpand(true);

    let title_lbl = Label::builder()
        .label(&book.title)
        .halign(gtk4::Align::Center)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .css_classes(vec!["title-3".to_string()])
        .build();

    let author_lbl = Label::builder()
        .label(&book.author)
        .halign(gtk4::Align::Center)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    page_header.append(&title_lbl);
    page_header.append(&author_lbl);

    top_bar.append(&back_btn);
    top_bar.append(&toggle_sidebar_btn);
    top_bar.append(&page_header);
    text_container.append(&top_bar);

    let text_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .build();

    let text_box = Box::new(Orientation::Vertical, 10);
    text_box.set_margin_start(24);
    text_box.set_margin_end(24);
    text_box.set_margin_top(16);
    text_box.set_margin_bottom(24);
    text_box.set_halign(gtk4::Align::Center);
    text_box.set_size_request(640, -1);

    let chapter_heading = Label::builder()
        .halign(gtk4::Align::Start)
        .wrap(true)
        .css_classes(vec!["title-1".to_string()])
        .build();

    let paragraphs_box = Box::new(Orientation::Vertical, 10);

    text_box.append(&chapter_heading);
    text_box.append(&paragraphs_box);
    text_scroll.set_child(Some(&text_box));
    text_container.append(&text_scroll);

    let chapters_ref: Rc<Vec<ebook_parsers::ParsedChapter>> = Rc::new(book.chapters.clone());
    let current_pages: Rc<RefCell<Vec<Vec<String>>>> = Rc::new(RefCell::new(Vec::new()));
    let loaded_chapter_idx: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

    let render_current_page = {
        let ch_idx_ref = current_chapter_idx.clone();
        let pg_idx_ref = current_page_idx.clone();
        let loaded_ch_ref = loaded_chapter_idx.clone();
        let pages_ref = current_pages.clone();
        let ch_list = chapters_ref.clone();
        let heading_ref = chapter_heading.clone();
        let p_box = paragraphs_box.clone();
        let scroll_ref = text_scroll.clone();
        let sb_slot = sidebar_slot.clone();
        let nav_s = nav_slot.clone();

        Rc::new(move || {
            let ch_idx = *ch_idx_ref.borrow();

            if *loaded_ch_ref.borrow() != Some(ch_idx) {
                let (title, content) = match ch_list.get(ch_idx) {
                    Some(ch) => (ch.title.as_str(), ch.content.as_str()),
                    None => ("Конец книги", "Больше нет страниц."),
                };

                heading_ref.set_label(title);
                *pages_ref.borrow_mut() = paginate_chapter_content(content);
                *loaded_ch_ref.borrow_mut() = Some(ch_idx);
            }

            if let Some(sb) = sb_slot.borrow().as_ref() {
                sb.highlight_chapter(ch_idx);
            }

            let pages = pages_ref.borrow();
            let total_pages = pages.len().max(1);
            let mut pg_idx = *pg_idx_ref.borrow();
            if pg_idx >= total_pages {
                pg_idx = total_pages - 1;
                *pg_idx_ref.borrow_mut() = pg_idx;
            }

            while let Some(child) = p_box.first_child() {
                p_box.remove(&child);
            }

            if let Some(page_paragraphs) = pages.get(pg_idx) {
                if pg_idx == 0 && page_paragraphs.is_empty() {
                    let empty_lbl = Label::builder()
                        .label("В этой главе нет текста.")
                        .halign(gtk4::Align::Start)
                        .css_classes(vec!["dim-label".to_string(), "body".to_string()])
                        .build();
                    p_box.append(&empty_lbl);
                } else {
                    for para in page_paragraphs {
                        let lbl = Label::builder()
                            .label(para)
                            .wrap(true)
                            .halign(gtk4::Align::Start)
                            .css_classes(vec!["body".to_string()])
                            .build();
                        p_box.append(&lbl);
                    }
                }
            }

            let remaining_minutes = estimate_remaining_minutes(&pages, pg_idx);

            if let Some(nav) = nav_s.borrow().as_ref() {
                nav.update(
                    pg_idx,
                    total_pages,
                    ch_idx,
                    total_chapters,
                    remaining_minutes,
                );
                nav.reset_scroll_and_show(&scroll_ref);
            }
        })
    };

    // Нижняя панель навигации по страницам
    let on_prev_page = {
        let pg_idx_ref = current_page_idx.clone();
        let render_cb = render_current_page.clone();
        Rc::new(move || {
            let current = *pg_idx_ref.borrow();
            if current > 0 {
                *pg_idx_ref.borrow_mut() = current - 1;
                render_cb();
            }
        })
    };

    let on_next_page = {
        let pg_idx_ref = current_page_idx.clone();
        let pages_ref = current_pages.clone();
        let render_cb = render_current_page.clone();
        Rc::new(move || {
            let current = *pg_idx_ref.borrow();
            let total_pages = pages_ref.borrow().len().max(1);
            if current + 1 < total_pages {
                *pg_idx_ref.borrow_mut() = current + 1;
                render_cb();
            }
        })
    };

    let nav_bar = Rc::new(ReaderNavBar::new(&text_scroll, on_prev_page, on_next_page));
    text_container.append(&nav_bar.widget);
    *nav_slot.borrow_mut() = Some(nav_bar);

    // Сайдбар читалки (оглавление + заметки)
    let on_chapter_selected = {
        let ch_idx_ref = current_chapter_idx.clone();
        let pg_idx_ref = current_page_idx.clone();
        let render_cb = render_current_page.clone();

        Rc::new(move |new_chapter_idx: usize| {
            *ch_idx_ref.borrow_mut() = new_chapter_idx;
            *pg_idx_ref.borrow_mut() = 0;
            render_cb();
        })
    };

    let reader_sidebar = Rc::new(ReaderSidebar::new(book, on_chapter_selected));
    let sidebar_widget = reader_sidebar.widget.clone();
    *sidebar_slot.borrow_mut() = Some(reader_sidebar);

    // Первоначальный рендер и подсветка первой главы
    render_current_page();

    root.append(&sidebar_widget);
    root.append(&text_container);
    root.upcast()
}
