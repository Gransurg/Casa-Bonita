use adw::prelude::*;
use gtk4::{
    Box, Button, Entry, Label, Orientation, PasswordEntry, ScrolledWindow, Separator, Widget,
};
use std::rc::Rc;

pub fn create_auth_view<F1, F2>(on_go_register: F1, on_continue_guest: F2) -> Widget
where
    F1: Fn() + 'static,
    F2: Fn() + 'static,
{
    let go_register = Rc::new(on_go_register);
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

    // Левая колонка зелёная панель 
    outer.append(&build_promo_panel(
        "ВАШЕ ПРОСТРАНСТВО ДЛЯ ЧТЕНИЯ",
        "Ваши книги.\nВаш ритм.",
        "Меньше шума. Больше хороших историй.",
        "Всё, что хочется прочитать.",
        "В одном спокойном месте.",
    ));

    // Правая колонка: форма входа 
    let right_col = Box::new(Orientation::Vertical, 14);
    right_col.set_size_request(380, -1);
    right_col.set_valign(gtk4::Align::Center);

    let heading = Label::builder()
        .label("С возвращением")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1".to_string()])
        .build();

    let subheading = Label::builder()
        .label("Войдите в аккаунт, чтобы вернуться\nк любимым историям.")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label".to_string()])
        .build();

    let email_lbl = Label::builder()
        .label("Электронная почта")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string()])
        .margin_top(6)
        .build();

    let email_entry = Entry::builder()
        .placeholder_text("reader@example.com")
        .css_classes(vec!["auth-input".to_string()])
        .build();

    let pass_header = Box::new(Orientation::Horizontal, 0);
    let pass_lbl = Label::builder()
        .label("Пароль")
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .css_classes(vec!["caption".to_string()])
        .build();
    let forgot_btn = Button::builder()
        .label("Забыли пароль?")
        .css_classes(vec!["flat".to_string(), "caption".to_string(), "auth-link-btn".to_string()])
        .build();
    pass_header.append(&pass_lbl);
    pass_header.append(&forgot_btn);

    let pass_entry = PasswordEntry::builder()
        .placeholder_text("••••••••••••")
        .show_peek_icon(true)
        .css_classes(vec!["auth-input".to_string()])
        .build();

    let login_btn = Button::builder()
        .label("Войти")
        .margin_top(6)
        .css_classes(vec!["auth-primary-btn".to_string()])
        .build();

    let cb_guest_login = continue_guest.clone();
    login_btn.connect_clicked(move |_| cb_guest_login());

    // Разделитель «или»
    let or_row = Box::new(Orientation::Horizontal, 12);
    or_row.set_margin_top(4);
    or_row.set_margin_bottom(4);
    let sep_left = Separator::new(Orientation::Horizontal);
    sep_left.set_hexpand(true);
    sep_left.set_valign(gtk4::Align::Center);
    let or_lbl = Label::builder()
        .label("или")
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();
    let sep_right = Separator::new(Orientation::Horizontal);
    sep_right.set_hexpand(true);
    sep_right.set_valign(gtk4::Align::Center);
    or_row.append(&sep_left);
    or_row.append(&or_lbl);
    or_row.append(&sep_right);

    let guest_btn = Button::builder()
        .label("Продолжить без аккаунта")
        .css_classes(vec!["auth-secondary-btn".to_string()])
        .build();
    let cb_guest = continue_guest;
    guest_btn.connect_clicked(move |_| cb_guest());

    let register_row = Box::new(Orientation::Horizontal, 4);
    register_row.set_halign(gtk4::Align::Center);
    register_row.set_margin_top(8);
    let no_acc_lbl = Label::builder()
        .label("Нет аккаунта?")
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();
    let create_acc_btn = Button::builder()
        .label("Создать аккаунт")
        .css_classes(vec!["flat".to_string(), "caption".to_string(), "auth-link-btn".to_string()])
        .build();
    let cb_reg = go_register;
    create_acc_btn.connect_clicked(move |_| cb_reg());
    register_row.append(&no_acc_lbl);
    register_row.append(&create_acc_btn);

    let footer_hint = Label::builder()
        .label("Книги на устройстве доступны и без входа.")
        .halign(gtk4::Align::Center)
        .margin_top(24)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    right_col.append(&heading);
    right_col.append(&subheading);
    right_col.append(&email_lbl);
    right_col.append(&email_entry);
    right_col.append(&pass_header);
    right_col.append(&pass_entry);
    right_col.append(&login_btn);
    right_col.append(&or_row);
    right_col.append(&guest_btn);
    right_col.append(&register_row);
    right_col.append(&footer_hint);

    outer.append(&right_col);
    scroll.set_child(Some(&outer));
    scroll.upcast()
}

pub(crate) fn build_promo_panel(
    kicker: &str,
    big_title: &str,
    big_sub: &str,
    bottom_title: &str,
    bottom_sub: &str,
) -> Widget {
    let card = Box::new(Orientation::Vertical, 12);
    card.set_size_request(420, 560);
    card.add_css_class("auth-promo-card");
    card.set_margin_start(8);
    card.set_margin_end(8);
    card.set_margin_top(8);
    card.set_margin_bottom(8);

    let top_box = Box::new(Orientation::Vertical, 8);
    top_box.set_margin_start(32);
    top_box.set_margin_end(32);
    top_box.set_margin_top(36);

    let kicker_lbl = Label::builder()
        .label(kicker)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    let title_lbl = Label::builder()
        .label(big_title)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1".to_string()])
        .build();

    let sub_lbl = Label::builder()
        .label(big_sub)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label".to_string()])
        .build();

    top_box.append(&kicker_lbl);
    top_box.append(&title_lbl);
    top_box.append(&sub_lbl);
    card.append(&top_box);

    // Мини-обложки 
    let covers_row = Box::new(Orientation::Horizontal, 10);
    covers_row.set_halign(gtk4::Align::Center);
    covers_row.set_valign(gtk4::Align::Center);
    covers_row.set_vexpand(true);

    let c1 = Box::new(Orientation::Vertical, 0);
    c1.set_size_request(92, 145);
    c1.set_valign(gtk4::Align::Center);
    c1.add_css_class("book-cover-terracotta");
    c1.append(&Label::builder().label("Сиддхартха").vexpand(true).valign(gtk4::Align::Center).build());

    let c2 = Box::new(Orientation::Vertical, 0);
    c2.set_size_request(112, 175);
    c2.set_valign(gtk4::Align::Center);
    c2.add_css_class("book-cover-green");
    c2.append(&Label::builder().label("Маленький\nпринц").justify(gtk4::Justification::Center).vexpand(true).valign(gtk4::Align::Center).build());

    let c3 = Box::new(Orientation::Vertical, 0);
    c3.set_size_request(92, 145);
    c3.set_valign(gtk4::Align::Center);
    c3.add_css_class("book-cover-slate");
    c3.append(&Label::builder().label("Письма\nпоэту").justify(gtk4::Justification::Center).vexpand(true).valign(gtk4::Align::Center).build());

    covers_row.append(&c1);
    covers_row.append(&c2);
    covers_row.append(&c3);
    card.append(&covers_row);

    let bottom_box = Box::new(Orientation::Vertical, 4);
    bottom_box.set_margin_start(32);
    bottom_box.set_margin_end(32);
    bottom_box.set_margin_bottom(32);

    bottom_box.append(&Separator::new(Orientation::Horizontal));

    let b_title = Label::builder()
        .label(bottom_title)
        .halign(gtk4::Align::Start)
        .margin_top(12)
        .css_classes(vec!["title-4".to_string()])
        .build();

    let b_sub = Label::builder()
        .label(bottom_sub)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
        .build();

    bottom_box.append(&b_title);
    bottom_box.append(&b_sub);
    card.append(&bottom_box);

    card.upcast()
}
