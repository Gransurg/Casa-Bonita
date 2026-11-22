pub mod error;
pub mod models;
pub mod services;

pub use error::{CoreError, CoreResult};
pub use models::{Book, BookFormat, Chapter, Note, ReadingProgress};
pub use services::{LibraryFilter, LibraryService, ReadingService};
