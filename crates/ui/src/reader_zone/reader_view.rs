use adw::prelude::*;
use ebook_parsers::ParsedBook;
use gtk4::{Box, Widget};
use std::cell::RefCell;
use std::rc::Rc;

use crate::pagination::{estimate_remaining_minutes, paginate_chapter_content};
use super::reader_layout::ReaderLayout;
use super::reader_nav::ReaderNavBar;
use super::reader_sidebar::ReaderSidebar;
use super::reader_state::ReaderState;

pub fn create_reader_view<F>(
    book: &ParsedBook,
    book_id: i64,
    db_conn: Rc<RefCell<rusqlite::Connection>>,
    on_back_to_shelf: F,
) -> Widget
where
    F: Fn() + 'static + Clone,
{
    let state = Rc::new(ReaderState::new(book, book_id, db_conn));
    let layout = ReaderLayout::new(&book.title, &book.author);

    let sidebar_slot: Rc<RefCell<Option<Rc<ReaderSidebar>>>> = Rc::new(RefCell::new(None));
    let nav_slot: Rc<RefCell<Option<Rc<ReaderNavBar>>>> = Rc::new(RefCell::new(None));

    {
        let cb_back = on_back_to_shelf;
        let state_back = state.clone();
        let pages_len_ref = Rc::new(RefCell::new(1_usize));

        layout.back_btn.connect_clicked(move |_| {
            state_back.save(*pages_len_ref.borrow());
            cb_back();
        });
    }

    {
        let sb_slot = sidebar_slot.clone();
        layout.toggle_sidebar_btn.connect_clicked(move |_| {
            if let Some(sb) = sb_slot.borrow().as_ref() {
                sb.toggle();
            }
        });
    }

    let chapters_ref: Rc<Vec<ebook_parsers::ParsedChapter>> = Rc::new(book.chapters.clone());
    let current_pages: Rc<RefCell<Vec<Vec<String>>>> = Rc::new(RefCell::new(Vec::new()));
    let loaded_chapter_idx: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

    {
        let ch = *state.current_chapter_idx.borrow();
        if ch < book.chapters.len() {
            let content = &book.chapters[ch].content;
            let computed_pages = paginate_chapter_content(content);
            *current_pages.borrow_mut() = computed_pages;
            *loaded_chapter_idx.borrow_mut() = Some(ch);
        }
    }

    let render_current_page = {
        let state_ref = state.clone();
        let loaded_ch_ref = loaded_chapter_idx.clone();
        let pages_ref = current_pages.clone();
        let ch_list = chapters_ref.clone();
        let heading_ref = layout.chapter_heading.clone();
        let p_box = layout.paragraphs_box.clone();
        let scroll_ref = layout.text_scroll.clone();
        let sb_slot = sidebar_slot.clone();
        let nav_s = nav_slot.clone();

        Rc::new(move || {
            let ch_idx = *state_ref.current_chapter_idx.borrow();

            if *loaded_ch_ref.borrow() != Some(ch_idx) {
                let (title, content) = match ch_list.get(ch_idx) {
                    Some(ch) => (ch.title.as_str(), ch.content.as_str()),
                    None => ("Конец книги", "Больше нет страниц."),
                };

                heading_ref.set_label(title);
                let computed = paginate_chapter_content(content);
                *pages_ref.borrow_mut() = computed;
                *loaded_ch_ref.borrow_mut() = Some(ch_idx);
            }

            if let Some(sb) = sb_slot.borrow().as_ref() {
                sb.highlight_chapter(ch_idx);
            }

            let pages = pages_ref.borrow();
            let total_pages = pages.len().max(1);
            let mut pg_idx = *state_ref.current_page_idx.borrow();
            if pg_idx >= total_pages {
                pg_idx = total_pages - 1;
                *state_ref.current_page_idx.borrow_mut() = pg_idx;
            }

            while let Some(child) = p_box.first_child() {
                p_box.remove(&child);
            }

            if let Some(page_paragraphs) = pages.get(pg_idx) {
                if pg_idx == 0 && page_paragraphs.is_empty() {
                    let empty_lbl = gtk4::Label::builder()
                        .label("В этой главе нет текста.")
                        .halign(gtk4::Align::Start)
                        .css_classes(vec!["dim-label".to_string(), "body".to_string()])
                        .build();
                    p_box.append(&empty_lbl);
                } else {
                    for para in page_paragraphs {
                        let lbl = gtk4::Label::builder()
                            .label(para)
                            .wrap(true)
                            .halign(gtk4::Align::Start)
                            .css_classes(vec!["body".to_string()])
                            .build();
                        p_box.append(&lbl);
                    }
                }
            }

            let remaining_minutes = estimate_remaining_minutes(&pages, pg_idx);

            if let Some(nav) = nav_s.borrow().as_ref() {
                nav.update(
                    pg_idx,
                    total_pages,
                    ch_idx,
                    state_ref.total_chapters,
                    remaining_minutes,
                );
                nav.reset_scroll_and_show(&scroll_ref);
            }

            state_ref.save(total_pages);
        })
    };

    let on_prev_page = {
        let state_ref = state.clone();
        let render_cb = render_current_page.clone();
        Rc::new(move || {
            let current = *state_ref.current_page_idx.borrow();
            if current > 0 {
                *state_ref.current_page_idx.borrow_mut() = current - 1;
                render_cb();
            }
        })
    };

    let on_next_page = {
        let state_ref = state.clone();
        let pages_ref = current_pages.clone();
        let render_cb = render_current_page.clone();
        Rc::new(move || {
            let current = *state_ref.current_page_idx.borrow();
            let total_pages = pages_ref.borrow().len().max(1);
            if current + 1 < total_pages {
                *state_ref.current_page_idx.borrow_mut() = current + 1;
                render_cb();
            }
        })
    };

    let nav_bar = Rc::new(ReaderNavBar::new(&layout.text_scroll, on_prev_page, on_next_page));
    
    // Добавляем навбар внутрь контейнера макета
    if let Some(text_container) = layout.root.first_child().and_downcast::<Box>() {
        text_container.append(&nav_bar.widget);
    }
    *nav_slot.borrow_mut() = Some(nav_bar);

    let on_chapter_selected = {
        let state_ref = state.clone();
        let render_cb = render_current_page.clone();

        Rc::new(move |new_chapter_idx: usize| {
            *state_ref.current_chapter_idx.borrow_mut() = new_chapter_idx;
            *state_ref.current_page_idx.borrow_mut() = 0;
            render_cb();
        })
    };

    let reader_sidebar = Rc::new(ReaderSidebar::new(book, on_chapter_selected));
    let sidebar_widget = reader_sidebar.widget.clone();
    *sidebar_slot.borrow_mut() = Some(reader_sidebar);

    render_current_page();

    layout.root.prepend(&sidebar_widget);
    layout.root.upcast()
}
