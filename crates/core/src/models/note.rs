#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: String,
    pub book_id: String,
    pub chapter_title: Option<String>,
    pub page: Option<u32>,
    pub content: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl Note {
    pub fn new(
        id: impl Into<String>,
        book_id: impl Into<String>,
        content: impl Into<String>,
        created_at: i64,
    ) -> Self {
        Self {
            id: id.into(),
            book_id: book_id.into(),
            chapter_title: None,
            page: None,
            content: content.into(),
            created_at,
            updated_at: None,
        }
    }
}
