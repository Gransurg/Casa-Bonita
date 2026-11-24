mod auth_view;
mod reader_view;
mod register_view;
mod sidebar;
mod views;
mod pagination;
mod reader_nav;
mod reader_sidebar;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use auth_view::create_auth_view;
use gtk4::gdk::Display;
use gtk4::{Button, CssProvider, Image, Orientation, Stack, Widget};
use register_view::create_register_view;
use std::cell::RefCell;
use std::rc::Rc;
use views::create_library_view;

pub fn run_app() -> adw::glib::ExitCode {
    let app = Application::builder()
        .application_id("com.github.methissedrac.list")
        .build();

    app.connect_startup(|_| load_css());

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("style.css"));

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(1200)
        .default_height(750)
        .title("Лист")
        .build();

    let root_box = gtk4::Box::new(Orientation::Horizontal, 0);

    // Сайдбар библиотеки
    let sidebar_container = gtk4::Box::new(Orientation::Vertical, 0);
    root_box.append(&sidebar_container);

    // Основная рабочая область
    let main_area = gtk4::Box::new(Orientation::Vertical, 0);
    main_area.set_hexpand(true);

    let views_stack = Stack::new();
    views_stack.set_vexpand(true);

    let header_bar = create_header_bar(
        sidebar_container.upcast_ref::<Widget>(),
        &views_stack,
        &sidebar_container,
    );
    main_area.append(&header_bar);

    // Состояние загруженных книг в памяти (опять же, заглушка до БД)
    let books: Rc<RefCell<Vec<ebook_parsers::ParsedBook>>> = Rc::new(RefCell::new(Vec::new()));

    // Функция перерисовки библиотеки
    let refresh_library: Rc<dyn Fn(Rc<dyn Fn(usize)>)> = {
        let stack_ref = views_stack.clone();
        let books_ref = books.clone();

        Rc::new(move |on_open_book_cb: Rc<dyn Fn(usize)>| {
            let b_list = books_ref.borrow();
            let library_view = create_library_view(&b_list, move |idx| {
                on_open_book_cb(idx);
            });

            if let Some(old_library) = stack_ref.child_by_name("library") {
                stack_ref.remove(&old_library);
            }

            stack_ref.add_named(&library_view, Some("library"));
            stack_ref.set_visible_child_name("library");
        })
    };

    // Обработчик открытия конкретной книги по её индексу
    let open_book_handler: Rc<RefCell<Option<Rc<dyn Fn(usize)>>>> = Rc::new(RefCell::new(None));
    {
        let stack_for_reader = views_stack.clone();
        let sidebar_for_reader = sidebar_container.clone();
        let books_for_reader = books.clone();
        let refresh_for_back = refresh_library.clone();
        let handler_weak = Rc::downgrade(&open_book_handler);

        *open_book_handler.borrow_mut() = Some(Rc::new(move |book_index: usize| {
            let b_list = books_for_reader.borrow();
            if let Some(book) = b_list.get(book_index) {
                let sidebar_b = sidebar_for_reader.clone();
                let ref_back = refresh_for_back.clone();
                let handler_weak_inner = handler_weak.clone();

                let reader = reader_view::create_reader_view(book, move || {
                    sidebar_b.set_visible(true);
                    if let Some(handler_rc) = handler_weak_inner.upgrade() {
                        if let Some(cb) = handler_rc.borrow().as_ref() {
                            ref_back(cb.clone());
                        }
                    }
                });

                if let Some(old_reader) = stack_for_reader.child_by_name("reader") {
                    stack_for_reader.remove(&old_reader);
                }

                stack_for_reader.add_named(&reader, Some("reader"));
                sidebar_for_reader.set_visible(false);
                stack_for_reader.set_visible_child_name("reader");
            }
        }));
    }

    let get_open_cb = {
        let handler_ref = open_book_handler.clone();
        move || handler_ref.borrow().as_ref().unwrap().clone()
    };

    // Первоначальная отрисовка библиотеки
    refresh_library(get_open_cb());

    // Колбэк при импорте книги через диалог в сайдбаре
    let books_for_add = books.clone();
    let refresh_for_add = refresh_library.clone();
    let get_open_for_add = get_open_cb.clone();
    let on_book_added = Rc::new(move |book: ebook_parsers::ParsedBook| {
        println!(
            "Добавлена книга: {} (глав: {})",
            book.title,
            book.chapters.len()
        );
        books_for_add.borrow_mut().push(book);
        refresh_for_add(get_open_for_add());
    });

    sidebar::create_sidebar(&sidebar_container, &window, on_book_added);

    // Экран авторизации
    let stack_for_reg = views_stack.clone();
    let sidebar_for_guest_auth = sidebar_container.clone();
    let refresh_for_guest_auth = refresh_library.clone();
    let get_open_for_guest_auth = get_open_cb.clone();
    let auth = create_auth_view(
        move || {
            stack_for_reg.set_visible_child_name("register");
        },
        move || {
            sidebar_for_guest_auth.set_visible(true);
            refresh_for_guest_auth(get_open_for_guest_auth());
        },
    );
    views_stack.add_named(&auth, Some("auth"));

    // Экран регистрации
    let stack_for_login = views_stack.clone();
    let sidebar_for_guest_reg = sidebar_container.clone();
    let refresh_for_guest_reg = refresh_library.clone();
    let get_open_for_guest_reg = get_open_cb.clone();
    let register = create_register_view(
        move || {
            stack_for_login.set_visible_child_name("auth");
        },
        move || {
            sidebar_for_guest_reg.set_visible(true);
            refresh_for_guest_reg(get_open_for_guest_reg());
        },
    );
    views_stack.add_named(&register, Some("register"));

    views_stack.set_visible_child_name("library");

    main_area.append(&views_stack);
    root_box.append(&main_area);
    window.set_content(Some(&root_box));
    window.present();
}

fn create_header_bar(
    sidebar: &Widget,
    views_stack: &Stack,
    sidebar_container: &gtk4::Box,
) -> HeaderBar {
    let header_bar = HeaderBar::new();

    let toggle_sidebar_btn = Button::builder()
        .icon_name("sidebar-show-symbolic")
        .tooltip_text("Боковая панель")
        .build();

    let sidebar_ref = sidebar.clone();
    toggle_sidebar_btn.connect_clicked(move |_| {
        let is_visible = sidebar_ref.is_visible();
        sidebar_ref.set_visible(!is_visible);
    });
    header_bar.pack_start(&toggle_sidebar_btn);

    // Иконка аккаунта по центру сверху
    let account_btn = Button::builder()
        .tooltip_text("Аккаунт")
        .css_classes(vec!["flat".to_string(), "circular".to_string()])
        .build();
    account_btn.set_child(Some(&Image::from_icon_name("avatar-default-symbolic")));

    let stack_ref = views_stack.clone();
    let sidebar_box_ref = sidebar_container.clone();
    account_btn.connect_clicked(move |_| {
        sidebar_box_ref.set_visible(false);
        stack_ref.set_visible_child_name("auth");
    });
    header_bar.set_title_widget(Some(&account_btn));

    let search_btn = Button::builder()
        .icon_name("system-search-symbolic")
        .tooltip_text("Поиск")
        .build();
    let filter_btn = Button::builder()
        .icon_name("view-filter-symbolic")
        .tooltip_text("Фильтры")
        .build();
    let bookmarks_btn = Button::builder()
        .icon_name("user-bookmarks-symbolic")
        .tooltip_text("Закладки")
        .build();
    let settings_btn = Button::builder()
        .icon_name("emblem-system-symbolic")
        .tooltip_text("Настройки")
        .build();

    header_bar.pack_end(&settings_btn);
    header_bar.pack_end(&bookmarks_btn);
    header_bar.pack_end(&filter_btn);
    header_bar.pack_end(&search_btn);

    header_bar
}
