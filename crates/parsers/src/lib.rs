use std::path::Path;
pub mod txt;
pub mod fb2;
pub use txt::TxtParser;
pub use fb2::Fb2Parser;
use ebook_core::{BookFormat, CoreResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedChapter {
    pub index: usize,
    pub title: String,
    pub href: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBook {
    pub title: String,
    pub author: String,
    pub language: Option<String>,
    pub format: BookFormat,
    pub chapters: Vec<ParsedChapter>,
}

pub trait BookParser {
    fn format(&self) -> BookFormat;
    fn can_parse(&self, path: &Path) -> bool;
    fn parse(&self, path: &Path) -> CoreResult<ParsedBook>;
}
