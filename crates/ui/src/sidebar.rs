use adw::prelude::*;
use adw::ApplicationWindow;
use ebook_parsers::{BookParser, Fb2Parser, ParsedBook, TxtParser};
use gtk4::{Button, Image, Label, ListBox, Orientation, PolicyType, ScrolledWindow, Widget};
use std::rc::Rc;

pub fn create_sidebar(
    sidebar_box: &gtk4::Box,
    window: &ApplicationWindow,
    on_book_added: Rc<dyn Fn(ParsedBook)>,
) -> Widget {
    sidebar_box.set_size_request(180, -1);
    sidebar_box.add_css_class("sidebar");

    // Верхняя панель сайдбара (название и кнопка скрытия)
    let top_bar = gtk4::Box::new(Orientation::Horizontal, 8);
    top_bar.set_margin_start(16);
    top_bar.set_margin_end(12);
    top_bar.set_margin_top(16);
    top_bar.set_margin_bottom(12);

    let app_title = Label::builder()
        .label("Лист")
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .css_classes(vec!["heading".to_string()])
        .build();
    top_bar.append(&app_title);

    let toggle_sidebar_btn = Button::builder()
        .icon_name("sidebar-show-symbolic")
        .tooltip_text("Свернуть панель")
        .css_classes(vec!["flat".to_string()])
        .build();

    let sidebar_box_clone = sidebar_box.clone();
    toggle_sidebar_btn.connect_clicked(move |_| {
        let is_visible = sidebar_box_clone.is_visible();
        sidebar_box_clone.set_visible(!is_visible);
    });
    top_bar.append(&toggle_sidebar_btn);
    sidebar_box.append(&top_bar);

    // Контент сайдбара
    let content_vbox = gtk4::Box::new(Orientation::Vertical, 12);
    content_vbox.set_margin_start(16);
    content_vbox.set_margin_end(16);
    content_vbox.set_margin_bottom(16);

    let lib_label = Label::builder()
        .label("МОЯ БИБЛИОТЕКА")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label".to_string(), "caption".to_string()])
        .margin_top(8)
        .build();
    content_vbox.append(&lib_label);

    let nav_list = ListBox::new();
    nav_list.set_css_classes(&["navigation-sidebar"]);

    let row_all = adw::ActionRow::builder().title("Все книги").subtitle("12").build();
    row_all.add_prefix(&Image::from_icon_name("book-open-symbolic"));

    let row_recent = adw::ActionRow::builder()
        .title("Недавно открытые")
        .subtitle("4")
        .build();
    row_recent.add_prefix(&Image::from_icon_name("document-open-recent-symbolic"));

    let row_fav = adw::ActionRow::builder().title("Избранное").subtitle("3").build();
    row_fav.add_prefix(&Image::from_icon_name("starred-symbolic"));

    let row_read = adw::ActionRow::builder().title("Прочитано").subtitle("5").build();
    row_read.add_prefix(&Image::from_icon_name("object-select-symbolic"));

    nav_list.append(&row_all);
    nav_list.append(&row_recent);
    nav_list.append(&row_fav);
    nav_list.append(&row_read);
    content_vbox.append(&nav_list);

    // Распорка для прижимания низа
    let spacer = gtk4::Box::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    content_vbox.append(&spacer);

    // Нижний блок: Кнопка добавления книги через GtkFileDialog
    let add_btn = Button::builder()
        .label("+ Добавить книгу")
        .css_classes(vec!["suggested-action".to_string(), "pill".to_string()])
        .build();

    let window_ref = window.clone();
    let cb_add = on_book_added;
    add_btn.connect_clicked(move |_| {
        let dialog = gtk4::FileDialog::builder()
            .title("Выберите книгу для импорта")
            .build();

        let win_clone = window_ref.clone();
        let cb_inner = cb_add.clone();

        dialog.open(Some(&win_clone), None::<&gtk4::gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let txt_parser = TxtParser::new();
                    let fb2_parser = Fb2Parser::new();

                    let parsed_book = if txt_parser.can_parse(&path) {
                        txt_parser.parse(&path)
                    } else if fb2_parser.can_parse(&path) {
                        fb2_parser.parse(&path)
                    } else {
                        eprintln!("Неподдерживаемый формат файла: {}", path.display());
                        return;
                    };

                    match parsed_book {
                        Ok(book) => {
                            println!("Успешно загружена книга: {} — {}", book.author, book.title);
                            cb_inner(book);
                        }
                        Err(err) => {
                            eprintln!("Ошибка разбора книги: {err}");
                        }
                    }
                }
            }
        });
    });

    content_vbox.append(&add_btn);

    let formats_label = Label::builder()
        .label("EPUB, FB2, TXT")
        .halign(gtk4::Align::Center)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();
    content_vbox.append(&formats_label);

    let stats_label = Label::builder()
        .label("12 книг · хранятся на устройстве")
        .halign(gtk4::Align::Center)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .margin_top(4)
        .build();
    content_vbox.append(&stats_label);

    let scrolled_sidebar = ScrolledWindow::builder()
        .child(&content_vbox)
        .hscrollbar_policy(PolicyType::Never)
        .vexpand(true)
        .build();

    sidebar_box.append(&scrolled_sidebar);
    sidebar_box.clone().upcast()
}
