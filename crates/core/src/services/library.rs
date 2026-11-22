use crate::models::{Book, BookFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryFilter {
    All,
    Favorites,
    Read,
    Recent,
    Format(BookFormat),
}

pub struct LibraryService;

impl LibraryService {
    /// Фильтрует список книг по выбранному фильтру и поисковому запросу.
    pub fn filter_books<'a>(
        books: &'a [Book],
        filter: LibraryFilter,
        query: &str,
    ) -> Vec<&'a Book> {
        let normalized_query = query.trim().to_lowercase();

        let mut filtered: Vec<&Book> = books
            .iter()
            .filter(|book| match filter {
                LibraryFilter::All => true,
                LibraryFilter::Favorites => book.is_favorite,
                LibraryFilter::Read => book.is_read,
                LibraryFilter::Recent => book.last_opened_at.is_some(),
                LibraryFilter::Format(fmt) => book.format == fmt,
            })
            .filter(|book| {
                if normalized_query.is_empty() {
                    return true;
                }
                book.title.to_lowercase().contains(&normalized_query)
                    || book.author.to_lowercase().contains(&normalized_query)
            })
            .collect();

        if matches!(filter, LibraryFilter::Recent) {
            filtered.sort_by(|a, b| b.last_opened_at.cmp(&a.last_opened_at));
        }

        filtered
    }

    pub fn toggle_favorite(book: &mut Book) {
        book.is_favorite = !book.is_favorite;
    }

    pub fn toggle_read(book: &mut Book) {
        book.is_read = !book.is_read;
    }

    pub fn mark_opened(book: &mut Book, opened_at: i64) {
        book.last_opened_at = Some(opened_at);
    }
}
