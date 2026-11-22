use adw::prelude::*;
use ebook_parsers::ParsedBook;
use gtk4::{
    Box, Button, GestureClick, Label, Orientation, PolicyType, Revealer,
    RevealerTransitionType, ScrolledWindow, Separator, Stack, ToggleButton, Widget,
};
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_reader_view<F>(book: &ParsedBook, on_back_to_shelf: F) -> Widget
where
    F: Fn() + 'static + Clone,
{
    let root = Box::new(Orientation::Horizontal, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let current_chapter_idx = Rc::new(RefCell::new(0));

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

    // Оборачиваем сайдбар в Revealer для независимого плавного скрытия/показа
    let toc_revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideRight)
        .transition_duration(250)
        .child(&sidebar)
        .reveal_child(true)
        .build();

    // Центральная область текста
    let text_container = Box::new(Orientation::Vertical, 0);
    text_container.set_hexpand(true);
    text_container.set_vexpand(true);

    let book_rc = Rc::new(book.clone());

    let render_chapter_content = {
        let container = text_container.clone();
        let b_ref = book_rc.clone();
        let idx_ref = current_chapter_idx.clone();
        let on_back_cb = on_back_to_shelf.clone();
        let revealer_ref = toc_revealer.clone();

        Rc::new(move || {
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }

            let idx = *idx_ref.borrow();
            let chapter = b_ref.chapters.get(idx).cloned().unwrap_or_else(|| {
                ebook_parsers::ParsedChapter {
                    index: 0,
                    title: "Конец книги".to_string(),
                    href: None,
                    content: "Больше нет страниц.".to_string(),
                }
            });

            // Верхняя плашка читалки
            let top_bar = Box::new(Orientation::Horizontal, 12);
            top_bar.set_margin_start(24);
            top_bar.set_margin_end(24);
            top_bar.set_margin_top(16);
            top_bar.set_margin_bottom(8);

            // Кнопка возврата на полку
            let back_btn = Button::builder()
                .label("← На полку")
                .css_classes(vec!["suggested-action".to_string(), "pill".to_string()])
                .valign(gtk4::Align::Center)
                .build();

            let cb = on_back_cb.clone();
            back_btn.connect_clicked(move |_| {
                cb();
            });

            // Отдельная кнопка переключения сайдбара с оглавлением и заметками
            let toggle_toc_btn = Button::builder()
                .icon_name("view-list-symbolic")
                .tooltip_text("Скрыть / показать оглавление")
                .css_classes(vec!["flat".to_string()])
                .valign(gtk4::Align::Center)
                .build();

            let rev_clone = revealer_ref.clone();
            toggle_toc_btn.connect_clicked(move |_| {
                let is_revealed = rev_clone.reveals_child();
                rev_clone.set_reveal_child(!is_revealed);
            });

            let page_header = Box::new(Orientation::Vertical, 2);
            page_header.set_halign(gtk4::Align::Center);
            page_header.set_hexpand(true);

            let title_lbl = Label::builder()
                .label(&b_ref.title)
                .halign(gtk4::Align::Center)
                .css_classes(vec!["title-3".to_string()])
                .build();

            let author_lbl = Label::builder()
                .label(&b_ref.author)
                .halign(gtk4::Align::Center)
                .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
                .build();

            page_header.append(&title_lbl);
            page_header.append(&author_lbl);

            // Балансирующий отступ справа для центрирования заголовка
            let right_balance = Box::new(Orientation::Horizontal, 0);
            right_balance.set_size_request(130, -1);

            top_bar.append(&back_btn);
            top_bar.append(&toggle_toc_btn);
            top_bar.append(&page_header);
            top_bar.append(&right_balance);
            container.append(&top_bar);

            // Текст главы
            let text_scroll = ScrolledWindow::builder()
                .hscrollbar_policy(PolicyType::Never)
                .vexpand(true)
                .build();

            let text_box = Box::new(Orientation::Vertical, 12);
            text_box.set_margin_start(64);
            text_box.set_margin_end(64);
            text_box.set_margin_top(16);
            text_box.set_margin_bottom(24);
            text_box.set_halign(gtk4::Align::Center);
            text_box.set_size_request(640, -1);

            let chapter_heading = Label::builder()
                .label(&chapter.title)
                .halign(gtk4::Align::Start)
                .css_classes(vec!["title-1".to_string()])
                .build();

            let chapter_text = Label::builder()
                .label(&chapter.content)
                .wrap(true)
                .halign(gtk4::Align::Start)
                .css_classes(vec!["body".to_string()])
                .build();

            text_box.append(&chapter_heading);
            text_box.append(&chapter_text);
            text_scroll.set_child(Some(&text_box));
            container.append(&text_scroll);
        })
    };

    // Первоначальный рендер страницы
    render_chapter_content();

    // Заполнение вкладок сайдбара читалки
    let toc_widget = build_toc_tab(
        book,
        current_chapter_idx.clone(),
        render_chapter_content.clone(),
    );
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

    // Нижняя мета-информация о формате книги
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

// Построение оглавления на базе кликабельных Box (гарантирует отображение текста поверх системных тем)
fn build_toc_tab(
    book: &ParsedBook,
    current_idx: Rc<RefCell<usize>>,
    on_chapter_selected: Rc<dyn Fn()>,
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
        let idx_ref = current_idx.clone();
        let cb_render = on_chapter_selected.clone();

        click_controller.connect_released(move |_, _, _, _| {
            *idx_ref.borrow_mut() = i;
            cb_render();
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
