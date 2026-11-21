mod auth_view;
mod reader_view;
mod register_view;
mod sidebar;
mod views;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use auth_view::create_auth_view;
use gtk4::gdk::Display;
use gtk4::{Button, CssProvider, Image, Orientation, Stack, Widget};
use reader_view::create_reader_view;
use register_view::create_register_view;
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
    sidebar::create_sidebar(&sidebar_container);
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

    let stack_for_open = views_stack.clone();
    let sidebar_for_open = sidebar_container.clone();
    let library = create_library_view(move || {
        sidebar_for_open.set_visible(false);
        stack_for_open.set_visible_child_name("reader");
    });
    views_stack.add_named(&library, Some("library"));

    let stack_for_back = views_stack.clone();
    let sidebar_for_back = sidebar_container.clone();
    let reader = create_reader_view(move || {
        sidebar_for_back.set_visible(true);
        stack_for_back.set_visible_child_name("library");
    });
    views_stack.add_named(&reader, Some("reader"));

    // Экран авторизации
    let stack_for_reg = views_stack.clone();
    let stack_for_guest_auth = views_stack.clone();
    let sidebar_for_guest_auth = sidebar_container.clone();
    let auth = create_auth_view(
        move || {
            stack_for_reg.set_visible_child_name("register");
        },
        move || {
            sidebar_for_guest_auth.set_visible(true);
            stack_for_guest_auth.set_visible_child_name("library");
        },
    );
    views_stack.add_named(&auth, Some("auth"));

    // Экран регистрации
    let stack_for_login = views_stack.clone();
    let stack_for_guest_reg = views_stack.clone();
    let sidebar_for_guest_reg = sidebar_container.clone();
    let register = create_register_view(
        move || {
            stack_for_login.set_visible_child_name("auth");
        },
        move || {
            sidebar_for_guest_reg.set_visible(true);
            stack_for_guest_reg.set_visible_child_name("library");
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
