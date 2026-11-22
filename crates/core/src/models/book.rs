use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookFormat {
    Txt,
    Epub,
    Fb2,
}

impl BookFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Epub => "EPUB",
            Self::Fb2 => "FB2",
            Self::Txt => "TXT",
        }
    }
}

impl fmt::Display for BookFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub language: Option<String>,
    pub format: BookFormat,
    pub file_path: PathBuf,
    pub is_favorite: bool,
    pub is_read: bool,
    pub added_at: i64,
    pub last_opened_at: Option<i64>,
}

impl Book {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        author: impl Into<String>,
        format: BookFormat,
        file_path: impl Into<PathBuf>,
        added_at: i64,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            author: author.into(),
            language: None,
            format,
            file_path: file_path.into(),
            is_favorite: false,
            is_read: false,
            added_at,
            last_opened_at: None,
        }
    }
}
