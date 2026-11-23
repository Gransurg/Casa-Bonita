use adw::prelude::*;
use ebook_parsers::ParsedBook;
use gtk4::{
    glib, Box, Button, GestureClick, Label, Orientation, PolicyType, Revealer,
    RevealerTransitionType, ScrolledWindow, Separator, Stack, ToggleButton, Widget,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

const PARAGRAPHS_PER_PAGE: usize = 40;
const SCROLL_IDLE_HIDE_MS: u64 = 250;

fn paginate_chapter_content(content: &str) -> Vec<Vec<String>> {
    let paragraphs: Vec<String> = content
        .split('\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();

    if paragraphs.is_empty() {
        return vec![Vec::new()];
    }

    paragraphs
        .chunks(PARAGRAPHS_PER_PAGE)
        .map(|chunk| chunk.to_vec())
        .collect()
}

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

    // --- Сайдбар читалки (Оглавление / Заметки) ---
    let sidebar = Box::new(Orientation::Vertical, 0);
    sidebar.set_size_request(240, -1);
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

    let toc_revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideRight)
        .transition_duration(250)
        .child(&sidebar)
        .reveal_child(true)
        .build();

    // --- Центральная область текста ---
    let text_container = Box::new(Orientation::Vertical, 0);
    text_container.set_hexpand(true);
    text_container.set_vexpand(true);

    let top_bar = Box::new(Orientation::Horizontal, 12);
    top_bar.set_margin_start(24);
    top_bar.set_margin_end(24);
    top_bar.set_margin_top(16);
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

    let toggle_toc_btn = Button::builder()
        .icon_name("view-list-symbolic")
        .tooltip_text("Скрыть / показать оглавление")
        .css_classes(vec!["flat".to_string()])
        .valign(gtk4::Align::Center)
        .build();

    let rev_clone = toc_revealer.clone();
    toggle_toc_btn.connect_clicked(move |_| {
        let is_revealed = rev_clone.reveals_child();
        rev_clone.set_reveal_child(!is_revealed);
    });

    let page_header = Box::new(Orientation::Vertical, 2);
    page_header.set_halign(gtk4::Align::Center);
    page_header.set_hexpand(true);

    let title_lbl = Label::builder()
        .label(&book.title)
        .halign(gtk4::Align::Center)
        .css_classes(vec!["title-3".to_string()])
        .build();

    let author_lbl = Label::builder()
        .label(&book.author)
        .halign(gtk4::Align::Center)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    page_header.append(&title_lbl);
    page_header.append(&author_lbl);

    let right_balance = Box::new(Orientation::Horizontal, 0);
    right_balance.set_size_request(130, -1);

    top_bar.append(&back_btn);
    top_bar.append(&toggle_toc_btn);
    top_bar.append(&page_header);
    top_bar.append(&right_balance);
    text_container.append(&top_bar);

    let text_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .build();

    let text_box = Box::new(Orientation::Vertical, 10);
    text_box.set_margin_start(64);
    text_box.set_margin_end(64);
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

    // --- Нижняя панель навигации по страницам текущей главы ---
    let nav_bar = Box::new(Orientation::Horizontal, 16);
    nav_bar.set_margin_start(24);
    nav_bar.set_margin_end(24);
    nav_bar.set_margin_top(8);
    nav_bar.set_margin_bottom(14);
    nav_bar.set_halign(gtk4::Align::Fill);

    let prev_page_btn = Button::builder()
        .label("← Предыдущая")
        .css_classes(vec!["pill".to_string()])
        .build();

    let page_counter_lbl = Label::builder()
        .hexpand(true)
        .halign(gtk4::Align::Center)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    let next_page_btn = Button::builder()
        .label("Следующая →")
        .css_classes(vec!["pill".to_string()])
        .build();

    nav_bar.append(&prev_page_btn);
    nav_bar.append(&page_counter_lbl);
    nav_bar.append(&next_page_btn);

    let nav_revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideUp)
        .transition_duration(180)
        .child(&nav_bar)
        .reveal_child(true)
        .build();

    text_container.append(&nav_revealer);

    // Скрываем стрелки во время прокрутки и возвращаем после паузы
    let scroll_generation = Rc::new(Cell::new(0_u64));
    let suppress_scroll_hide = Rc::new(Cell::new(false));

    {
        let rev = nav_revealer.clone();
        let gen_cell = scroll_generation.clone();
        let suppress = suppress_scroll_hide.clone();

        text_scroll.vadjustment().connect_value_changed(move |_| {
            if suppress.get() {
                return;
            }

            rev.set_reveal_child(false);
            let next_gen = gen_cell.get().wrapping_add(1);
            gen_cell.set(next_gen);

            let rev_timer = rev.clone();
            let gen_timer = gen_cell.clone();
            glib::timeout_add_local_once(
                Duration::from_millis(SCROLL_IDLE_HIDE_MS),
                move || {
                    if gen_timer.get() == next_gen {
                        rev_timer.set_reveal_child(true);
                    }
                },
            );
        });
    }

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
        let prev_btn = prev_page_btn.clone();
        let next_btn = next_page_btn.clone();
        let counter_lbl = page_counter_lbl.clone();
        let scroll_ref = text_scroll.clone();
        let nav_rev = nav_revealer.clone();
        let suppress = suppress_scroll_hide.clone();

        Rc::new(move || {
            let ch_idx = *ch_idx_ref.borrow();

            if *loaded_ch_ref.borrow() != Some(ch_idx) {
                let (title, content) = match ch_list.get(ch_idx) {
                    Some(ch) => (ch.title.as_str(), ch.content.as_str()),
                    None => ("Конец книги", "Больше нет страниц."),
                };

                heading_ref.set_label(title);
                let paginated = paginate_chapter_content(content);
                *pages_ref.borrow_mut() = paginated;
                *loaded_ch_ref.borrow_mut() = Some(ch_idx);
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

            prev_btn.set_sensitive(pg_idx > 0);
            next_btn.set_sensitive(pg_idx + 1 < total_pages);

            if total_chapters > 0 {
                counter_lbl.set_label(&format!(
                    "Стр. {} из {} · Глава {} из {}",
                    pg_idx + 1,
                    total_pages,
                    ch_idx + 1,
                    total_chapters
                ));
            } else {
                counter_lbl.set_label(&format!("Стр. {} из {}", pg_idx + 1, total_pages));
            }

            suppress.set(true);
            scroll_ref.vadjustment().set_value(0.0);
            suppress.set(false);
            nav_rev.set_reveal_child(true);
        })
    };

    // Листаем только страницы внутри выбранной главы
    {
        let pg_idx_ref = current_page_idx.clone();
        let render_cb = render_current_page.clone();
        prev_page_btn.connect_clicked(move |_| {
            let current = *pg_idx_ref.borrow();
            if current > 0 {
                *pg_idx_ref.borrow_mut() = current - 1;
                render_cb();
            }
        });
    }

    {
        let pg_idx_ref = current_page_idx.clone();
        let pages_ref = current_pages.clone();
        let render_cb = render_current_page.clone();
        next_page_btn.connect_clicked(move |_| {
            let current = *pg_idx_ref.borrow();
            let total_pages = pages_ref.borrow().len().max(1);
            if current + 1 < total_pages {
                *pg_idx_ref.borrow_mut() = current + 1;
                render_cb();
            }
        });
    }

    // Выбор главы происходит только из оглавления
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

    // Первоначальный рендер первой страницы первой главы
    render_current_page();

    // Вкладки оглавления и заметок
    let toc_widget = build_toc_tab(book, on_chapter_selected);
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
    sidebar_bottom.set_margin_start(14);
    sidebar_bottom.set_margin_end(14);
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

    root.append(&toc_revealer);
    root.append(&text_container);
    root.upcast()
}

fn build_toc_tab(
    book: &ParsedBook,
    on_chapter_selected: Rc<dyn Fn(usize)>,
) -> Widget {
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .build();

    let list = Box::new(Orientation::Vertical, 4);
    list.set_margin_start(8);
    list.set_margin_end(8);
    list.set_margin_top(10);
    list.set_margin_bottom(10);

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
    }

    scroll.set_child(Some(&list));
    scroll.upcast()
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
