use crate::auth_view::build_promo_panel;
use adw::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, PasswordEntry, ScrolledWindow, Widget};
use std::rc::Rc;

pub fn create_register_view<F1, F2>(on_go_login: F1, on_continue_guest: F2) -> Widget
where
    F1: Fn() + 'static,
    F2: Fn() + 'static,
{
    let go_login = Rc::new(on_go_login);
    let continue_guest = Rc::new(on_continue_guest);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .build();

    let outer = Box::new(Orientation::Horizontal, 48);
    outer.set_margin_start(36);
    outer.set_margin_end(36);
    outer.set_margin_top(24);
    outer.set_margin_bottom(24);
    outer.set_halign(gtk4::Align::Center);
    outer.set_valign(gtk4::Align::Center);

    // Левая колонка в том же стиле 
    outer.append(&build_promo_panel(
        "НОВЫЙ ЧИТАТЕЛЬ",
        "Своя полка.\nБез лишнего.",
        "Сохраняйте заметки, прогресс и книги в одном месте.",
        "Начните читать сейчас.",
        "Аккаунт можно создать за пару секунд.",
    ));

    // Правая колонка: форма регистрации 
    let right_col = Box::new(Orientation::Vertical, 14);
    right_col.set_size_request(380, -1);
    right_col.set_valign(gtk4::Align::Center);

    let heading = Label::builder()
        .label("Создать аккаунт")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1".to_string()])
        .build();

    let subheading = Label::builder()
        .label("Зарегистрируйтесь, чтобы синхронизировать\nзаметки и прогресс чтения.")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label".to_string()])
        .build();

    let name_lbl = Label::builder()
        .label("Имя или псевдоним")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string()])
        .margin_top(6)
        .build();

    let name_entry = Entry::builder()
        .placeholder_text("Анна")
        .css_classes(vec!["auth-input".to_string()])
        .build();

    let email_lbl = Label::builder()
        .label("Электронная почта")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string()])
        .build();

    let email_entry = Entry::builder()
        .placeholder_text("reader@example.com")
        .css_classes(vec!["auth-input".to_string()])
        .build();

    let pass_lbl = Label::builder()
        .label("Пароль")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string()])
        .build();

    let pass_entry = PasswordEntry::builder()
        .placeholder_text("Не менее 8 символов")
        .show_peek_icon(true)
        .css_classes(vec!["auth-input".to_string()])
        .build();

    let submit_btn = Button::builder()
        .label("Создать аккаунт")
        .margin_top(6)
        .css_classes(vec!["auth-primary-btn".to_string()])
        .build();
    let cb_done = continue_guest.clone();
    submit_btn.connect_clicked(move |_| cb_done());

    let guest_btn = Button::builder()
        .label("Продолжить без аккаунта")
        .css_classes(vec!["auth-secondary-btn".to_string()])
        .build();
    let cb_guest = continue_guest;
    guest_btn.connect_clicked(move |_| cb_guest());

    let login_row = Box::new(Orientation::Horizontal, 4);
    login_row.set_halign(gtk4::Align::Center);
    login_row.set_margin_top(8);
    let has_acc_lbl = Label::builder()
        .label("Уже есть аккаунт?")
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();
    let go_login_btn = Button::builder()
        .label("Войти")
        .css_classes(vec!["flat".to_string(), "caption".to_string(), "auth-link-btn".to_string()])
        .build();
    let cb_login = go_login;
    go_login_btn.connect_clicked(move |_| cb_login());
    login_row.append(&has_acc_lbl);
    login_row.append(&go_login_btn);

    right_col.append(&heading);
    right_col.append(&subheading);
    right_col.append(&name_lbl);
    right_col.append(&name_entry);
    right_col.append(&email_lbl);
    right_col.append(&email_entry);
    right_col.append(&pass_lbl);
    right_col.append(&pass_entry);
    right_col.append(&submit_btn);
    right_col.append(&guest_btn);
    right_col.append(&login_row);

    outer.append(&right_col);
    scroll.set_child(Some(&outer));
    scroll.upcast()
}
