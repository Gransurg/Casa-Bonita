use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    BookNotFound(String),
    ChapterNotFound(String),
    NoteNotFound(String),
    InvalidProgress(String),
    InvalidInput(String),
    StorageError(String),
    ParseError(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BookNotFound(id) => write!(f, "Книга не найдена: {id}"),
            Self::ChapterNotFound(id) => write!(f, "Глава не найдена: {id}"),
            Self::NoteNotFound(id) => write!(f, "Заметка не найдена: {id}"),
            Self::InvalidProgress(msg) => write!(f, "Некорректный прогресс чтения: {msg}"),
            Self::InvalidInput(msg) => write!(f, "Некорректные данные: {msg}"),
            Self::StorageError(msg) => write!(f, "Ошибка хранилища: {msg}"),
            Self::ParseError(msg) => write!(f, "Ошибка разбора книги: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}

pub type CoreResult<T> = Result<T, CoreError>;
