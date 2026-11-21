use adw::prelude::*;
use gtk4::{
    Box, Button, Image, Label, Orientation, PolicyType, ProgressBar, ScrolledWindow, Separator,
    Stack, ToggleButton, Widget,
};

pub fn create_reader_view<F>(on_back_to_shelf: F) -> Widget
where
    F: Fn() + 'static,
{
    let root = Box::new(Orientation::Horizontal, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    // --- Левый сайдбар читалки ---
    let sidebar = Box::new(Orientation::Vertical, 0);
    sidebar.set_size_request(220, -1);
    sidebar.add_css_class("sidebar");

    // Переключатель двух вкладок: «Оглавление» и «Заметки»
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

    // Содержимое вкладок через Stack
    let stack = Stack::new();
    stack.set_vexpand(true);
    stack.add_named(&build_toc_tab(), Some("toc"));
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

    // Низ сайдбара: формат и язык книги
    let sidebar_bottom = Box::new(Orientation::Vertical, 4);
    sidebar_bottom.set_margin_start(14);
    sidebar_bottom.set_margin_end(14);
    sidebar_bottom.set_margin_top(8);
    sidebar_bottom.set_margin_bottom(14);

    let meta_label = Label::builder()
        .label("EPUB · Русский")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    sidebar_bottom.append(&meta_label);
    sidebar.append(&sidebar_bottom);

    root.append(&sidebar);

    // --- Центральная часть страницы ---
    let main_page = Box::new(Orientation::Vertical, 0);
    main_page.set_hexpand(true);
    main_page.set_vexpand(true);

    // Верхняя панель страницы: «← На полку» слева и название/автор по центру
    let top_bar = Box::new(Orientation::Horizontal, 12);
    top_bar.set_margin_start(24);
    top_bar.set_margin_end(24);
    top_bar.set_margin_top(16);
    top_bar.set_margin_bottom(8);

    let back_btn = Button::builder()
        .label("← На полку")
        .css_classes(vec!["flat".to_string(), "filter-pill".to_string()])
        .valign(gtk4::Align::Center)
        .build();

    back_btn.connect_clicked(move |_| {
        on_back_to_shelf();
    });

    let page_header = Box::new(Orientation::Vertical, 2);
    page_header.set_halign(gtk4::Align::Center);
    page_header.set_hexpand(true);

    let book_title = Label::builder()
        .label("Маленький принц")
        .halign(gtk4::Align::Center)
        .css_classes(vec!["title-3".to_string()])
        .build();

    let book_author = Label::builder()
        .label("Антуан де Сент-Экзюпери")
        .halign(gtk4::Align::Center)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    page_header.append(&book_title);
    page_header.append(&book_author);

    // Невидимый баланс справа, чтобы заголовок оставался строго по центру
    let right_balance = Box::new(Orientation::Horizontal, 0);
    right_balance.set_size_request(90, -1);

    top_bar.append(&back_btn);
    top_bar.append(&page_header);
    top_bar.append(&right_balance);
    main_page.append(&top_bar);

    // Короткая заглушка текста страницы
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
        .label("Глава II")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1".to_string()])
        .build();

    let sample_text = Label::builder()
        .label(
            "Так я жил в одиночестве, и не с кем было мне поговорить по душам.\n\n\
             Пожалуйста... нарисуй мне барашка!",
        )
        .wrap(true)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["body".to_string()])
        .build();

    text_box.append(&chapter_heading);
    text_box.append(&sample_text);
    text_scroll.set_child(Some(&text_box));
    main_page.append(&text_scroll);

    // Нижняя строка навигации по странице (кнопка панели — чуть выше прогресс-бара)
    main_page.append(&build_bottom_bar(&sidebar));

    root.append(&main_page);
    root.upcast()
}

fn build_toc_tab() -> Widget {
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .build();

    let list = Box::new(Orientation::Vertical, 4);
    list.set_margin_start(12);
    list.set_margin_end(12);
    list.set_margin_top(14);
    list.set_margin_bottom(12);

    // (название/номер главы, прочитана ли, текущая ли)
    let chapters = [
        ("Посвящение", true, false),
        ("Глава I", true, false),
        ("Глава II", false, true),
        ("Глава III", false, false),
        ("Глава IV", false, false),
    ];

    for (title, is_done, is_current) in chapters {
        let row = Box::new(Orientation::Horizontal, 8);
        row.add_css_class("chapter-row");
        if is_current {
            row.add_css_class("chapter-row-current");
        }

        let title_lbl = Label::builder()
            .label(title)
            .halign(gtk4::Align::Start)
            .build();
        row.append(&title_lbl);

        // Надпись «Сейчас» чуть правее номера/названия главы
        if is_current {
            let now_lbl = Label::builder()
                .label("Сейчас")
                .halign(gtk4::Align::Start)
                .css_classes(vec!["chapter-badge-now".to_string()])
                .build();
            row.append(&now_lbl);
        }

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        row.append(&spacer);

        // Галочка для прочитанных глав
        if is_done {
            let check = Image::from_icon_name("object-select-symbolic");
            check.add_css_class("dim-label");
            row.append(&check);
        }

        list.append(&row);
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

    // (номер главы, текст заметки, страница)
    let notes = [
        ("Глава II", "Нарисовать барашка в ящике.", "Стр. 9"),
        ("Глава I", "Взрослые любят цифры.", "Стр. 5"),
    ];

    for (chapter, text, page) in notes {
        let card = Box::new(Orientation::Vertical, 4);
        card.add_css_class("note-card");

        let chapter_lbl = Label::builder()
            .label(chapter)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string(), "heading".to_string()])
            .build();

        let text_lbl = Label::builder()
            .label(text)
            .wrap(true)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["body".to_string()])
            .build();

        let page_lbl = Label::builder()
            .label(page)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        card.append(&chapter_lbl);
        card.append(&text_lbl);
        card.append(&page_lbl);
        list.append(&card);
    }

    scroll.set_child(Some(&list));
    scroll.upcast()
}

fn build_bottom_bar(sidebar: &Box) -> Widget {
    let container = Box::new(Orientation::Vertical, 6);
    container.set_margin_start(24);
    container.set_margin_end(24);
    container.set_margin_bottom(14);

    // Кнопка открытия/скрытия панели — чуть выше прогресс-бара, не исчезает при скрытии
    let panel_btn_row = Box::new(Orientation::Horizontal, 0);
    panel_btn_row.set_halign(gtk4::Align::Start);

    let toggle_panel_btn = Button::new();
    let toggle_label = Label::new(Some("Скрыть панель"));
    let btn_content = Box::new(Orientation::Horizontal, 6);
    btn_content.append(&Image::from_icon_name("sidebar-show-symbolic"));
    btn_content.append(&toggle_label);
    toggle_panel_btn.set_child(Some(&btn_content));
    toggle_panel_btn.add_css_class("flat");
    toggle_panel_btn.add_css_class("filter-pill");

    let sidebar_ref = sidebar.clone();
    let label_ref = toggle_label.clone();
    toggle_panel_btn.connect_clicked(move |_| {
        let next_visible = !sidebar_ref.is_visible();
        sidebar_ref.set_visible(next_visible);
        if next_visible {
            label_ref.set_label("Скрыть панель");
        } else {
            label_ref.set_label("Открыть панель");
        }
    });

    panel_btn_row.append(&toggle_panel_btn);
    container.append(&panel_btn_row);

    // Полоска прогресса чтения
    let mini_bar = ProgressBar::builder()
        .fraction(0.08)
        .css_classes(vec!["mini-book-progress".to_string()])
        .build();
    container.append(&mini_bar);

    let bar = Box::new(Orientation::Horizontal, 16);

    // Левее снизу: % прочитанной книги и время до конца главы
    let stats_label = Label::builder()
        .label("8% книги   ·   До конца главы ~4 мин")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    let left_spacer = Box::new(Orientation::Horizontal, 0);
    left_spacer.set_hexpand(true);

    // По центру снизу: стрелки и текущая страница / всего страниц
    let nav_box = Box::new(Orientation::Horizontal, 12);
    nav_box.set_halign(gtk4::Align::Center);

    let prev_btn = Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(vec!["flat".to_string(), "circular".to_string()])
        .build();

    let page_counter = Label::builder()
        .label("9 / 112")
        .valign(gtk4::Align::Center)
        .css_classes(vec!["caption".to_string()])
        .build();

    let next_btn = Button::builder()
        .icon_name("go-next-symbolic")
        .css_classes(vec!["flat".to_string(), "circular".to_string()])
        .build();

    nav_box.append(&prev_btn);
    nav_box.append(&page_counter);
    nav_box.append(&next_btn);

    let right_spacer = Box::new(Orientation::Horizontal, 0);
    right_spacer.set_hexpand(true);

    bar.append(&stats_label);
    bar.append(&left_spacer);
    bar.append(&nav_box);
    bar.append(&right_spacer);

    container.append(&bar);
    container.upcast()
}
