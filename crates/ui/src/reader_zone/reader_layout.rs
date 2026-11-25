use adw::prelude::*;
use gtk4::{Box, Button, Label, Orientation, PolicyType, ScrolledWindow, Widget};

pub struct ReaderLayout {
    pub root: Box,
    pub back_btn: Button,
    pub toggle_sidebar_btn: Button,
    pub chapter_heading: Label,
    pub paragraphs_box: Box,
    pub text_scroll: ScrolledWindow,
}

impl ReaderLayout {
    pub fn new(title: &str, author: &str) -> Self {
        let root = Box::new(Orientation::Horizontal, 0);
        root.set_hexpand(true);
        root.set_vexpand(true);

        let text_container = Box::new(Orientation::Vertical, 0);
        text_container.set_hexpand(true);
        text_container.set_vexpand(true);

        let top_bar = Box::new(Orientation::Horizontal, 12);
        top_bar.set_margin_start(20);
        top_bar.set_margin_end(20);
        top_bar.set_margin_top(14);
        top_bar.set_margin_bottom(8);

        let back_btn = Button::builder()
            .label("← На полку")
            .css_classes(vec!["suggested-action".to_string(), "pill".to_string()])
            .valign(gtk4::Align::Center)
            .build();

        let toggle_sidebar_btn = Button::builder()
            .icon_name("sidebar-show-symbolic")
            .tooltip_text("Скрыть / открыть боковую панель")
            .css_classes(vec!["flat".to_string()])
            .valign(gtk4::Align::Center)
            .build();

        let page_header = Box::new(Orientation::Vertical, 2);
        page_header.set_halign(gtk4::Align::Center);
        page_header.set_hexpand(true);

        let title_lbl = Label::builder()
            .label(title)
            .halign(gtk4::Align::Center)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .css_classes(vec!["title-3".to_string()])
            .build();

        let author_lbl = Label::builder()
            .label(author)
            .halign(gtk4::Align::Center)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        page_header.append(&title_lbl);
        page_header.append(&author_lbl);

        top_bar.append(&back_btn);
        top_bar.append(&toggle_sidebar_btn);
        top_bar.append(&page_header);
        text_container.append(&top_bar);

        let text_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .hexpand(true)
            .build();

        let text_box = Box::new(Orientation::Vertical, 10);
        text_box.set_margin_start(24);
        text_box.set_margin_end(24);
        text_box.set_margin_top(16);
        text_box.set_margin_bottom(24);
        text_box.set_halign(gtk4::Align::Center);
        text_box.set_size_request(640, -1);

        let chapter_heading = Label::builder()
            .halign(gtk4::Align::Start)
            .wrap(true)
            .css_classes(vec!["title-1".to_string()])
            .build();

        let paragraphs_box = Box::new(Orientation::Vertical, 10);

        text_box.append(&chapter_heading);
        text_box.append(&paragraphs_box);
        text_scroll.set_child(Some(&text_box));
        text_container.append(&text_scroll);

        root.append(&text_container);

        Self {
            root,
            back_btn,
            toggle_sidebar_btn,
            chapter_heading,
            paragraphs_box,
            text_scroll,
        }
    }
}
