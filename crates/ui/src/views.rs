use adw::prelude::*;
use gtk4::{
    Box, Button, Label, Orientation, PolicyType, ProgressBar, ScrolledWindow, ToggleButton,
    Widget,
};
use std::rc::Rc;

pub fn create_library_view<F>(on_open_book: F) -> Widget
where
    F: Fn() + 'static,
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

    // Верхний блок: Приветствие и баннер «Сейчас читаете» 
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

    let banner = create_reading_banner(open_book.clone());
    content_box.append(&banner);

    // Секция «На вашей полке» 
    let shelf_section = create_shelf_section(open_book);
    content_box.append(&shelf_section);

    // Распорка, чтобы прижать нижнюю строку к низу окна
    let bottom_spacer = Box::new(Orientation::Vertical, 0);
    bottom_spacer.set_vexpand(true);
    content_box.append(&bottom_spacer);

    // Нижняя панель статистики 
    let footer_box = create_footer_bar();
    content_box.append(&footer_box);

    scrolled.set_child(Some(&content_box));
    scrolled.upcast()
}

fn create_reading_banner(on_open_book: Rc<dyn Fn()>) -> Widget {
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
        .label("Маленький\nпринц")
        .justify(gtk4::Justification::Center)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .vexpand(true)
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
        .label("Маленький принц")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-2".to_string()])
        .build();

    let author_label = Label::builder()
        .label("Антуан де Сент-Экзюпери")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label".to_string()])
        .build();

    let progress_bar = ProgressBar::builder()
        .fraction(0.08)
        .margin_top(8)
        .margin_end(48)
        .css_classes(vec!["reading-progress".to_string()])
        .build();

    let progress_label = Label::builder()
        .label("Глава II · 8% прочитано")
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

    let cb = on_open_book.clone();
    continue_btn.connect_clicked(move |_| {
        cb();
    });

    let hint_label = Label::builder()
        .label("Ваше место сохранено автоматически")
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .margin_top(8)
        .halign(gtk4::Align::End)
        .build();

    action_box.append(&continue_btn);
    action_box.append(&hint_label);
    banner_box.append(&action_box);

    banner_box.upcast()
}

fn create_shelf_section(on_open_book: Rc<dyn Fn()>) -> Widget {
    let section_box = Box::new(Orientation::Vertical, 16);

    let header_box = Box::new(Orientation::Horizontal, 12);
    header_box.set_valign(gtk4::Align::Center);

    let shelf_title = Label::builder()
        .label("На вашей полке")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-3".to_string()])
        .build();

    let count_label = Label::builder()
        .label("12 книг")
        .valign(gtk4::Align::Baseline)
        .css_classes(vec!["dim-label".to_string()])
        .build();

    let header_spacer = Box::new(Orientation::Horizontal, 0);
    header_spacer.set_hexpand(true);

    let filter_group = Box::new(Orientation::Horizontal, 0);
    filter_group.add_css_class("linked");

    let sort_date_btn = ToggleButton::builder()
        .label("По дате")
        .active(true)
        .css_classes(vec!["filter-pill".to_string()])
        .build();

    let sort_title_btn = ToggleButton::builder()
        .label("По названию")
        .group(&sort_date_btn)
        .css_classes(vec!["filter-pill".to_string()])
        .build();

    filter_group.append(&sort_date_btn);
    filter_group.append(&sort_title_btn);

    header_box.append(&shelf_title);
    header_box.append(&count_label);
    header_box.append(&header_spacer);
    header_box.append(&filter_group);
    section_box.append(&header_box);

    let books_grid = Box::new(Orientation::Horizontal, 18);

    // Активная карточка «Маленький принц»
    books_grid.append(&create_book_card(
        "Маленький принц",
        "Антуан де Сент-Экзюпери",
        "8% · Глава II",
        "book-cover-green",
        Some(on_open_book),
    ));

    // Остальные пока заглушки без перехода
    books_grid.append(&create_book_card(
        "Сиддхартха",
        "Герман Гессе",
        "34% · У реки",
        "book-cover-terracotta",
        None,
    ));
    books_grid.append(&create_book_card(
        "Письма к молодому поэту",
        "Р. М. Рильке",
        "12% · Письмо второе",
        "book-cover-slate",
        None,
    ));
    books_grid.append(&create_book_card(
        "Сад расходящихся тропок",
        "Хорхе Луис Борхес",
        "Не начато",
        "book-cover-sand",
        None,
    ));

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

fn create_footer_bar() -> Widget {
    let footer_box = Box::new(Orientation::Horizontal, 12);
    footer_box.set_margin_top(12);

    let offline_hint = Label::builder()
        .label("Все книги доступны офлайн")
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    let shown_count = Label::builder()
        .label("Доступно 6 из 12 книг")
        .halign(gtk4::Align::End)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    footer_box.append(&offline_hint);
    footer_box.append(&shown_count);

    footer_box.upcast()
}
