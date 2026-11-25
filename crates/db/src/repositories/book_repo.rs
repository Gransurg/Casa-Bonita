use rusqlite::{params, Connection, Result};
use ebook_core::{Book, BookFormat};
use std::path::PathBuf;

pub struct BookRepository;

impl BookRepository {
    /// Сохранить новую книгу в базу данных
    pub fn insert_book(conn: &Connection, book: &Book) -> Result<()> {
        let format_str = match book.format {
            BookFormat::Txt => "TXT",
            BookFormat::Fb2 => "FB2",
            BookFormat::Epub => "EPUB",
        };

        conn.execute(
            r#"
            INSERT OR IGNORE INTO books (title, author, file_path, format, is_favorite, is_finished, added_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                book.title,
                book.author,
                book.file_path.to_str().unwrap_or(""),
                format_str,
                if book.is_favorite { 1 } else { 0 },
                if book.is_read { 1 } else { 0 },
                book.added_at,
            ],
        )?;
        Ok(())
    }

    /// Получить список всех сохраненных книг из базы
    pub fn get_all_books(conn: &Connection) -> Result<Vec<Book>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, author, file_path, format, is_favorite, is_finished, added_at FROM books"
        )?;

        let book_iter = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let author: Option<String> = row.get(2)?;
            let file_path_str: String = row.get(3)?;
            let format_str: String = row.get(4)?;
            let is_favorite: i32 = row.get(5)?;
            let is_finished: i32 = row.get(6)?;
            let added_at: i64 = row.get(7)?;

            let format = match format_str.to_uppercase().as_str() {
                "FB2" => BookFormat::Fb2,
                "EPUB" => BookFormat::Epub,
                _ => BookFormat::Txt,
            };

            Ok(Book {
                id: id.to_string(),
                title,
                author: author.unwrap_or_else(|| "Неизвестный автор".to_string()),
                language: None,
                format,
                file_path: PathBuf::from(file_path_str),
                is_favorite: is_favorite == 1,
                is_read: is_finished == 1,
                added_at,
                last_opened_at: None,
            })
        })?;

        let mut books = Vec::new();
        for book in book_iter {
            books.push(book?);
        }

        Ok(books)
    }
}
