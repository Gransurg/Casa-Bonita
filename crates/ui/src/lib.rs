mod auth_view;
mod register_view;
mod sidebar;
mod views;
mod pagination;
pub mod reader_zone;
use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use auth_view::create_auth_view;
use ebook_parsers::BookParser;
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

    let db_path = std::path::Path::new("list.db");
    let db_conn = match ebook_db::init_database(db_path) {
        Ok(conn) => {
            println!("База данных успешно инициализирована: {}", db_path.display());
            Rc::new(RefCell::new(conn))
        }
        Err(err) => {
            eprintln!("Ошибка инициализации базы данных: {err}");
            std::process::exit(1);
        }
    };

    let sidebar_container = gtk4::Box::new(Orientation::Vertical, 0);
    root_box.append(&sidebar_container);

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

    let conn_for_books = db_conn.clone();
    let load_books_from_db = move || -> Vec<(i64, ebook_parsers::ParsedBook)> {
        let conn = conn_for_books.borrow();
        let mut stmt = conn.prepare("SELECT book_id, last_position, percentage FROM reading_progress").unwrap();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, f64>(2)?))
        }).unwrap();
                
        for r in rows {
            println!("DEBUG DB PROGRESS: {:?}", r);
        }
        match ebook_db::BookRepository::get_all_books(&conn) {
            Ok(book_models) => {
                let mut parsed_books = Vec::new();
                for m in book_models {
                    let file_path_str = m.file_path.to_string_lossy();
                    let book_id: i64 = match conn.query_row(
                        "SELECT id FROM books WHERE file_path = ?1",
                        rusqlite::params![file_path_str.as_ref()],
                        |row| row.get(0),
                    ) {
                        Ok(id) => id,
                        Err(err) => {
                            eprintln!(
                                "Не удалось получить id книги по пути {}: {err}",
                                m.file_path.display()
                            );
                            continue;
                        }
                    };

                    let parser = ebook_parsers::Fb2Parser::new();
                    if let Ok(parsed) = parser.parse(&m.file_path) {
                        parsed_books.push((book_id, parsed));
                    } else {
                        let txt_parser = ebook_parsers::TxtParser::new();
                        if let Ok(parsed) = txt_parser.parse(&m.file_path) {
                            parsed_books.push((book_id, parsed));
                        }
                    }
                }
                parsed_books
            }
            Err(err) => {
                eprintln!("Ошибка загрузки книг из базы: {err}");
                Vec::new()
            }
        }
    };

    let books: Rc<RefCell<Vec<(i64, ebook_parsers::ParsedBook)>>> = Rc::new(RefCell::new(load_books_from_db()));

    let refresh_library: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

    let db_conn_for_del = db_conn.clone();
    let books_for_del = books.clone();
    let window_for_del = window.clone();

    let open_book_handler: Rc<RefCell<Option<Rc<dyn Fn(usize)>>>> = Rc::new(RefCell::new(None));
    let get_open_cb = {
        let handler_ref = open_book_handler.clone();
        move || handler_ref.borrow().as_ref().unwrap().clone()
    };

    // Клонируем Rc заранее, чтобы избежать перемещения оригинального refresh_library в замыкание
    let refresh_library_for_del = refresh_library.clone();

    let on_delete_book = move |book_id: i64| {
        let window_ref = window_for_del.clone();
        let db_conn_ref = db_conn_for_del.clone();
        let books_ref = books_for_del.clone();
        let refresh_cell = refresh_library_for_del.clone();

        let dialog = gtk4::MessageDialog::new(
            Some(&window_ref),
            gtk4::DialogFlags::MODAL,
            gtk4::MessageType::Warning,
            gtk4::ButtonsType::OkCancel,
            "Удалить книгу?",
        );
        dialog.set_secondary_text(Some(
            "Вы действительно хотите удалить эту книгу из библиотеки? Прогресс чтения также будет удален.",
        ));

        dialog.connect_response(move |dlg, response| {
            if response == gtk4::ResponseType::Ok {
                let conn = db_conn_ref.borrow();
                let _ = conn.execute("DELETE FROM books WHERE id = ?1", rusqlite::params![book_id]);
                let _ = conn.execute("DELETE FROM reading_progress WHERE book_id = ?1", rusqlite::params![book_id]);
                drop(conn);

                books_ref.borrow_mut().retain(|(id, _)| *id != book_id);
                
                if let Some(refresh_fn) = refresh_cell.borrow().as_ref() {
                    refresh_fn();
                }
            }
            dlg.close();
        });

        dialog.show();
    };

    let refresh_library_fn = {
        let stack_ref = views_stack.clone();
        let books_ref = books.clone();
        let db_conn_for_views = db_conn.clone();
        let del_cb = on_delete_book.clone();

        Rc::new(move |on_open_book_cb: Rc<dyn Fn(usize)>| {
            let b_list = books_ref.borrow();

            let mut active_idx: Option<usize> = None;
            let mut active_progress = None;

            {
                let conn = db_conn_for_views.borrow();
                let mut stmt = match conn
                    .prepare("SELECT book_id FROM reading_progress ORDER BY last_read_at DESC") {
                        Ok(s) => s,
                        Err(_) => return,
                    };

                let ids_iter = stmt.query_map([], |row| row.get::<_, i64>(0)).unwrap();

                for id_res in ids_iter {
                    let Ok(b_id) = id_res else { continue };
                    if let Some(pos) = b_list.iter().position(|(id, _)| *id == b_id) {
                        active_idx = Some(pos);

                        if let Ok(Some((last_pos, percentage, _))) =
                            ebook_db::ProgressRepository::get_progress(&conn, b_id)
                        {
                            let parts: Vec<&str> = last_pos.split(':').collect();
                            let chapter_idx = if parts.len() == 2 {
                                parts[0].parse::<usize>().unwrap_or(0)
                            } else {
                                0
                            };
                            active_progress = Some((percentage, chapter_idx as i32));
                        }
                        break;
                    }
                }
            }

            let active_idx = active_idx.unwrap_or(0);
            let del_cb_inner = del_cb.clone();

            let library_view = create_library_view(
                &b_list,
                active_progress,
                active_idx, 
                move |idx| {
                    on_open_book_cb(idx);
                },
                move |id| {
                    del_cb_inner(id);
                },
            );

            if let Some(old_library) = stack_ref.child_by_name("library") {
                stack_ref.remove(&old_library);
            }

            stack_ref.add_named(&library_view, Some("library"));
            stack_ref.set_visible_child_name("library");
        })
    };

    *refresh_library.borrow_mut() = {
        let ref_for_cell = refresh_library_fn.clone();
        let get_cb_for_cell = get_open_cb.clone();
        Some(Rc::new(move || {
            ref_for_cell(get_cb_for_cell());
        }))
    };

    {
        let stack_for_reader = views_stack.clone();
        let sidebar_for_reader = sidebar_container.clone();
        let books_for_reader = books.clone();
        let ref_cell_for_reader = refresh_library.clone();
        let handler_weak = Rc::downgrade(&open_book_handler);
        let db_conn_for_open = db_conn.clone();

        *open_book_handler.borrow_mut() = Some(Rc::new(move |book_index: usize| {
            let b_list = books_for_reader.borrow();
            if let Some((book_id, book)) = b_list.get(book_index) {
                let sidebar_b = sidebar_for_reader.clone();
                let ref_cell_inner = ref_cell_for_reader.clone();
                let handler_weak_inner = handler_weak.clone();
                let db_conn_reader = db_conn_for_open.clone();

                let reader = reader_zone::create_reader_view(book, *book_id, db_conn_reader, move || {
                    sidebar_b.set_visible(true);
                    if let Some(refresh_rc) = ref_cell_inner.borrow().as_ref() {
                        refresh_rc();
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

    refresh_library_fn(get_open_cb());

    let books_for_add = books.clone();
    let conn_for_add = db_conn.clone();
    let ref_cell_for_add = refresh_library.clone();
    let on_book_added = Rc::new(move |(book, file_path): (ebook_parsers::ParsedBook, std::path::PathBuf)| {
        println!(
            "Добавлена книга: {} (глав: {})",
            book.title,
            book.chapters.len()
        );

        let book_uuid = uuid::Uuid::new_v4().to_string();
        let new_book_model = ebook_core::Book::new(
            book_uuid,
            book.title.clone(),
            book.author.clone(),
            book.format,
            file_path,
            chrono::Utc::now().timestamp(),
        );

        let conn = conn_for_add.borrow();
        let mut inserted_id = 0_i64;
        if let Err(err) = ebook_db::BookRepository::insert_book(&conn, &new_book_model) {
            eprintln!("Не удалось сохранить книгу в базу данных: {err}");
        } else {
            inserted_id = conn.last_insert_rowid();
        }

        books_for_add.borrow_mut().push((inserted_id, book));
        if let Some(refresh_rc) = ref_cell_for_add.borrow().as_ref() {
            refresh_rc();
        }
    });

    sidebar::create_sidebar(&sidebar_container, &window, on_book_added);

    let stack_for_reg = views_stack.clone();
    let sidebar_for_guest_auth = sidebar_container.clone();
    let ref_cell_for_guest_auth = refresh_library.clone();
    let auth = create_auth_view(
        move || {
            stack_for_reg.set_visible_child_name("register");
        },
        move || {
            sidebar_for_guest_auth.set_visible(true);
            if let Some(refresh_rc) = ref_cell_for_guest_auth.borrow().as_ref() {
                refresh_rc();
            }
        },
    );
    views_stack.add_named(&auth, Some("auth"));

    let stack_for_login = views_stack.clone();
    let sidebar_for_guest_reg = sidebar_container.clone();
    let ref_cell_for_guest_reg = refresh_library.clone();
    let register = create_register_view(
        move || {
            stack_for_login.set_visible_child_name("auth");
        },
        move || {
            sidebar_for_guest_reg.set_visible(true);
            if let Some(refresh_rc) = ref_cell_for_guest_reg.borrow().as_ref() {
                refresh_rc();
            }
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
