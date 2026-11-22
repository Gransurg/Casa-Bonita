#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    pub id: String,
    pub book_id: String,
    pub index: usize,
    pub title: String,
    pub href: Option<String>,
    pub is_done: bool,
    pub is_current: bool,
}

impl Chapter {
    pub fn new(
        id: impl Into<String>,
        book_id: impl Into<String>,
        index: usize,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            book_id: book_id.into(),
            index,
            title: title.into(),
            href: None,
            is_done: false,
            is_current: false,
        }
    }
}
