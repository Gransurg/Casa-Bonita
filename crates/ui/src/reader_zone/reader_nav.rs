use adw::prelude::*;
use gtk4::{
    glib, Box, Button, Label, Orientation, Revealer, RevealerTransitionType, ScrolledWindow,
    Widget,
};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

const SCROLL_IDLE_HIDE_MS: u64 = 250;

pub struct ReaderNavBar {
    pub widget: Widget,
    revealer: Revealer,
    prev_btn: Button,
    next_btn: Button,
    counter_lbl: Label,
    suppress_scroll_hide: Rc<Cell<bool>>,
}

impl ReaderNavBar {
    pub fn new(
        scroll: &ScrolledWindow,
        on_prev_page: Rc<dyn Fn()>,
        on_next_page: Rc<dyn Fn()>,
    ) -> Self {
        let nav_bar = Box::new(Orientation::Horizontal, 16);
        nav_bar.set_margin_start(24);
        nav_bar.set_margin_end(24);
        nav_bar.set_margin_top(8);
        nav_bar.set_margin_bottom(14);
        nav_bar.set_halign(gtk4::Align::Fill);

        let prev_btn = Button::builder()
            .label("← Предыдущая")
            .css_classes(vec!["pill".to_string()])
            .build();

        let counter_lbl = Label::builder()
            .hexpand(true)
            .halign(gtk4::Align::Center)
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        let next_btn = Button::builder()
            .label("Следующая →")
            .css_classes(vec!["pill".to_string()])
            .build();

        nav_bar.append(&prev_btn);
        nav_bar.append(&counter_lbl);
        nav_bar.append(&next_btn);

        let revealer = Revealer::builder()
            .transition_type(RevealerTransitionType::SlideUp)
            .transition_duration(180)
            .child(&nav_bar)
            .reveal_child(true)
            .build();

        prev_btn.connect_clicked(move |_| {
            on_prev_page();
        });

        next_btn.connect_clicked(move |_| {
            on_next_page();
        });

        let scroll_generation = Rc::new(Cell::new(0_u64));
        let suppress_scroll_hide = Rc::new(Cell::new(false));

        {
            let rev = revealer.clone();
            let gen_cell = scroll_generation.clone();
            let suppress = suppress_scroll_hide.clone();

            scroll.vadjustment().connect_value_changed(move |_| {
                if suppress.get() {
                    return;
                }

                rev.set_reveal_child(false);
                let next_gen = gen_cell.get().wrapping_add(1);
                gen_cell.set(next_gen);

                let rev_timer = rev.clone();
                let gen_timer = gen_cell.clone();
                glib::timeout_add_local_once(
                    Duration::from_millis(SCROLL_IDLE_HIDE_MS),
                    move || {
                        if gen_timer.get() == next_gen {
                            rev_timer.set_reveal_child(true);
                        }
                    },
                );
            });
        }

        let widget = revealer.clone().upcast();

        Self {
            widget,
            revealer,
            prev_btn,
            next_btn,
            counter_lbl,
            suppress_scroll_hide,
        }
    }

    pub fn update(
        &self,
        page_idx: usize,
        total_pages: usize,
        chapter_idx: usize,
        total_chapters: usize,
        remaining_minutes: usize,
    ) {
        self.prev_btn.set_sensitive(page_idx > 0);
        self.next_btn.set_sensitive(page_idx + 1 < total_pages);

        let time_part = if remaining_minutes == 0 {
            String::new()
        } else if remaining_minutes == 1 {
            " · ~1 мин до конца главы".to_string()
        } else {
            format!(" · ~{} мин до конца главы", remaining_minutes)
        };

        if total_chapters > 0 {
            self.counter_lbl.set_label(&format!(
                "Стр. {} из {} · Глава {} из {}{}",
                page_idx + 1,
                total_pages,
                chapter_idx + 1,
                total_chapters,
                time_part
            ));
        } else {
            self.counter_lbl.set_label(&format!(
                "Стр. {} из {}{}",
                page_idx + 1,
                total_pages,
                time_part
            ));
        }
    }

    pub fn reset_scroll_and_show(&self, scroll: &ScrolledWindow) {
        self.suppress_scroll_hide.set(true);
        scroll.vadjustment().set_value(0.0);
        self.suppress_scroll_hide.set(false);
        self.revealer.set_reveal_child(true);
    }
}
