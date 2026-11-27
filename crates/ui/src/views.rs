use adw::prelude::*;
use ebook_parsers::ParsedBook;
use gtk4::{
    Box, Button, Label, Orientation, PolicyType, ProgressBar, ScrolledWindow,
    Widget,
};
use std::rc::Rc;

pub fn create_library_view<F>(
    books: &[ParsedBook],
    active_book_progress: Option<(f64, i32)>,
    active_book_idx: usize,
    on_open_book: F,
) -> Widget
where
    F: Fn(usize) + 'static,
{
    let open_book = Rc::new(on_open_book);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .build();

    let content_box = Box::new(Orientation::Vertical, 24);
    content_box.set_margin_start(32);
    content_box.set_margin_end(32);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);

    let welcome_box = Box::new(Orientation::Vertical, 6);
    let title_label = Label::builder()
        .label("Хорошего чтения")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1".to_string()])
        .build();
    let subtitle_label = Label::builder()
        .label("Ваша следующая история уже здесь.")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label".to_string()])
        .build();

    welcome_box.append(&title_label);
    welcome_box.append(&subtitle_label);
    content_box.append(&welcome_box);

    if let Some(book) = books.get(active_book_idx) {
        let cb_banner = open_book.clone();
        let idx = active_book_idx;
        let banner = create_reading_banner(book, active_book_progress, Rc::new(move || {
            cb_banner(idx); 
        }));
        content_box.append(&banner);
    } else if let Some(first_book) = books.first() {
        let cb_banner = open_book.clone();
        let banner = create_reading_banner(first_book, active_book_progress, Rc::new(move || {
            cb_banner(0);
        }));
        content_box.append(&banner);
    }

    let shelf_section = create_shelf_section(books, open_book);
    content_box.append(&shelf_section);

    let bottom_spacer = Box::new(Orientation::Vertical, 0);
    bottom_spacer.set_vexpand(true);
    content_box.append(&bottom_spacer);

    let footer_box = create_footer_bar(books.len());
    content_box.append(&footer_box);

    scrolled.set_child(Some(&content_box));
    scrolled.upcast()
}

fn create_reading_banner(
    book: &ParsedBook,
    progress_opt: Option<(f64, i32)>,
    on_open_book: Rc<dyn Fn()>,
) -> Widget {
    let banner_box = Box::new(Orientation::Horizontal, 24);
    banner_box.set_margin_top(4);
    banner_box.set_margin_bottom(4);
    banner_box.set_size_request(-1, 150);
    banner_box.add_css_class("reading-banner");
    banner_box.set_valign(gtk4::Align::Center);

    let cover_mock = Box::new(Orientation::Vertical, 0);
    cover_mock.set_size_request(95, 135);
    cover_mock.add_css_class("book-cover-green");
    cover_mock.set_margin_start(20);
    cover_mock.set_margin_top(16);
    cover_mock.set_margin_bottom(16);

    let cover_label = Label::builder()
        .label(&book.title)
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .vexpand(true)
        .margin_start(8)
        .margin_end(8)
        .build();
    cover_mock.append(&cover_label);
    banner_box.append(&cover_mock);

    let info_box = Box::new(Orientation::Vertical, 6);
    info_box.set_valign(gtk4::Align::Center);
    info_box.set_hexpand(true);

    let status_tag = Label::builder()
        .label("СЕЙЧАС ЧИТАЕТЕ")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    let book_title = Label::builder()
        .label(&book.title)
        .halign(gtk4::Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .css_classes(vec!["title-2".to_string()])
        .build();

    let author_label = Label::builder()
        .label(&book.author)
        .halign(gtk4::Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .css_classes(vec!["dim-label".to_string()])
        .build();

    let (percentage_fraction, progress_text) = if let Some((perc, ch)) = progress_opt {
        let display_ch = (ch as usize + 1).min(book.chapters.len());
        (
            perc / 100.0,
            format!("Глава {} из {} · {:.1}%", display_ch, book.chapters.len(), perc),
        )
    } else {
        (
            0.0,
            format!("Глав: {} · Не начато", book.chapters.len()),
        )
    };

    let progress_bar = ProgressBar::builder()
        .fraction(percentage_fraction)
        .margin_top(8)
        .margin_end(48)
        .css_classes(vec!["reading-progress".to_string()])
        .build();

    let progress_label = Label::builder()
        .label(&progress_text)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .margin_top(2)
        .build();

    info_box.append(&status_tag);
    info_box.append(&book_title);
    info_box.append(&author_label);
    info_box.append(&progress_bar);
    info_box.append(&progress_label);
    banner_box.append(&info_box);

    let action_box = Box::new(Orientation::Vertical, 0);
    action_box.set_valign(gtk4::Align::Center);
    action_box.set_margin_end(24);

    let continue_btn = Button::builder()
        .label("Продолжить чтение  →")
        .css_classes(vec!["suggested-action".to_string(), "pill".to_string()])
        .build();

    continue_btn.connect_clicked(move |_| {
        on_open_book();
    });

    let hint_label = Label::builder()
        .label("Место сохранено автоматически")
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .margin_top(8)
        .halign(gtk4::Align::End)
        .build();

    action_box.append(&continue_btn);
    action_box.append(&hint_label);
    banner_box.append(&action_box);

    banner_box.upcast()
}

fn create_shelf_section(books: &[ParsedBook], on_open_book: Rc<dyn Fn(usize)>) -> Widget {
    let section_box = Box::new(Orientation::Vertical, 16);

    let header_box = Box::new(Orientation::Horizontal, 12);
    header_box.set_valign(gtk4::Align::Center);

    let shelf_title = Label::builder()
        .label("На вашей полке")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-3".to_string()])
        .build();

    let count_label = Label::builder()
        .label(&format!("{} книг", books.len()))
        .valign(gtk4::Align::Baseline)
        .css_classes(vec!["dim-label".to_string()])
        .build();

    let header_spacer = Box::new(Orientation::Horizontal, 0);
    header_spacer.set_hexpand(true);

    header_box.append(&shelf_title);
    header_box.append(&count_label);
    header_box.append(&header_spacer);
    section_box.append(&header_box);

    let books_grid = Box::new(Orientation::Horizontal, 18);

    if books.is_empty() {
        let empty_lbl = Label::builder()
            .label("Пока нет добавленных книг")
            .css_classes(vec!["dim-label".to_string()])
            .build();
        books_grid.append(&empty_lbl);
    } else {
        let styles = ["book-cover-green", "book-cover-terracotta", "book-cover-slate", "book-cover-sand"];
        for (i, book) in books.iter().enumerate() {
            let style = styles[i % styles.len()];
            let cb_open = on_open_book.clone();
            let card = create_book_card(
                &book.title,
                &book.author,
                &format!("Глав: {}", book.chapters.len()),
                style,
                Some(Rc::new(move || {
                    cb_open(i);
                })),
            );
            books_grid.append(&card);
        }
    }

    section_box.append(&books_grid);
    section_box.upcast()
}

fn create_book_card(
    title: &str,
    author: &str,
    progress: &str,
    cover_css: &str,
    on_click: Option<Rc<dyn Fn()>>,
) -> Widget {
    let card_box = Box::new(Orientation::Vertical, 8);
    card_box.set_size_request(160, -1);

    let cover = Box::new(Orientation::Vertical, 0);
    cover.set_size_request(160, 220);
    cover.add_css_class(cover_css);

    let title_in_cover = Label::builder()
        .label(title)
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .vexpand(true)
        .margin_start(14)
        .margin_end(14)
        .build();
    cover.append(&title_in_cover);
    card_box.append(&cover);

    let t_label = Label::builder()
        .label(title)
        .halign(gtk4::Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .css_classes(vec!["heading".to_string()])
        .build();

    let a_label = Label::builder()
        .label(author)
        .halign(gtk4::Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .css_classes(vec!["dim-label".to_string(), "caption".to_string()])
        .build();

    let p_label = Label::builder()
        .label(progress)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    card_box.append(&t_label);
    card_box.append(&a_label);
    card_box.append(&p_label);

    if let Some(cb) = on_click {
        let btn = Button::builder()
            .child(&card_box)
            .css_classes(vec!["flat".to_string()])
            .build();
        btn.connect_clicked(move |_| {
            cb();
        });
        btn.upcast()
    } else {
        card_box.upcast()
    }
}

fn create_footer_bar(count: usize) -> Widget {
    let footer_box = Box::new(Orientation::Horizontal, 12);
    footer_box.set_margin_top(12);

    let offline_hint = Label::builder()
        .label("Все книги доступны офлайн")
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    let shown_count = Label::builder()
        .label(&format!("Доступно книг на устройстве: {}", count))
        .halign(gtk4::Align::End)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    footer_box.append(&offline_hint);
    footer_box.append(&shown_count);

    footer_box.upcast()
}
