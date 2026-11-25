use ebook_parsers::ParsedBook;
use std::cell::RefCell;
use std::rc::Rc;
use rusqlite::Connection;

pub struct ReaderState {
    pub book_id: i64,
    pub db_conn: Rc<RefCell<Connection>>,
    pub current_chapter_idx: Rc<RefCell<usize>>,
    pub current_page_idx: Rc<RefCell<usize>>,
    pub total_chapters: usize,
}

impl ReaderState {
    pub fn new(book: &ParsedBook, book_id: i64, db_conn: Rc<RefCell<Connection>>) -> Self {
        let mut initial_chapter = 0_usize;
        let mut initial_page = 0_usize;

        {
            let conn = db_conn.borrow();
            if let Ok(Some((last_pos, _, _))) = ebook_db::ProgressRepository::get_progress(&conn, book_id) {
                let parts: Vec<&str> = last_pos.split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(ch), Ok(pg)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                        if ch < book.chapters.len() {
                            initial_chapter = ch;
                            initial_page = pg;
                        }
                    }
                }
            }
        }

        Self {
            book_id,
            db_conn,
            current_chapter_idx: Rc::new(RefCell::new(initial_chapter)),
            current_page_idx: Rc::new(RefCell::new(initial_page)),
            total_chapters: book.chapters.len(),
        }
    }

    pub fn save(&self, total_pages: usize) {
        let ch = *self.current_chapter_idx.borrow();
        let pg = *self.current_page_idx.borrow();
        let pos_str = format!("{}:{}", ch, pg);
        let pages_len = total_pages.max(1);
        let percentage = ((pg as f64 + 1.0) / (pages_len as f64)) * 100.0;

        let conn = self.db_conn.borrow();
        let _ = ebook_db::ProgressRepository::save_progress(
            &conn,
            self.book_id,
            &pos_str,
            percentage,
            0,
        );
    }
}
