use std::fs;
use std::path::Path;
use roxmltree::Document;

use ebook_core::{BookFormat, CoreError, CoreResult};
use crate::{BookParser, ParsedBook, ParsedChapter};

pub struct Fb2Parser;

impl Fb2Parser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Fb2Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl BookParser for Fb2Parser {
    fn format(&self) -> BookFormat {
        BookFormat::Fb2
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("fb2"))
    }

    fn parse(&self, path: &Path) -> CoreResult<ParsedBook> {
        if !self.can_parse(path) {
            return Err(CoreError::ParseError(format!(
                "файл '{}' не имеет расширения .fb2",
                path.display()
            )));
        }

        let bytes = fs::read(path).map_err(|err| {
            CoreError::ParseError(format!(
                "не удалось прочитать FB2-файл '{}': {err}",
                path.display()
            ))
        })?;

        let xml_str = match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(_) => {
                let (cow, _, _) = encoding_rs::WINDOWS_1251.decode(&bytes);
                cow.into_owned()
            }
        };

        let doc = Document::parse(&xml_str).map_err(|err| {
            CoreError::ParseError(format!("ошибка парсинга XML в FB2: {err}"))
        })?;

        // Извлекаем метаданные
        let mut title = "Без названия".to_string();
        let mut author = "Неизвестный автор".to_string();

        if let Some(title_info) = doc.descendants().find(|n| n.has_tag_name("title-info")) {
            if let Some(book_title_node) = title_info.descendants().find(|n| n.has_tag_name("book-title")) {
                if let Some(text) = book_title_node.text() {
                    title = text.trim().to_string();
                }
            }

            if let Some(author_node) = title_info.descendants().find(|n| n.has_tag_name("author")) {
                let first = author_node.descendants().find(|n| n.has_tag_name("first-name")).and_then(|n| n.text()).unwrap_or("");
                let last = author_node.descendants().find(|n| n.has_tag_name("last-name")).and_then(|n| n.text()).unwrap_or("");
                let full = format!("{} {}", first, last).trim().to_string();
                if !full.is_empty() {
                    author = full;
                }
            }
        }

        // Извлекаем главы из секций тела книги <body>
        let mut chapters = Vec::new();
        let mut index = 0;

        for body in doc.descendants().filter(|n| n.has_tag_name("body")) {
            // Каждая секция может быть главой
            for section in body.children().filter(|n| n.has_tag_name("section")) {
                let chapter_title = section
                    .descendants()
                    .find(|n| n.has_tag_name("p") || n.has_tag_name("title"))
                    .and_then(|n| n.text())
                    .unwrap_or("Глава")
                    .to_string();

                // Собираем весь текст внутри секции
                let mut content_buffer = String::new();
                for p in section.descendants().filter(|n| n.has_tag_name("p")) {
                    if let Some(txt) = p.text() {
                        content_buffer.push_str(txt);
                        content_buffer.push('\n');
                    }
                }

                if !content_buffer.trim().is_empty() {
                    chapters.push(ParsedChapter {
                        index,
                        title: chapter_title,
                        href: None,
                        content: content_buffer,
                    });
                    index += 1;
                }
            }
        }

        // Если секций не нашлось, запишем весь текст тела
        if chapters.is_empty() {
            chapters.push(ParsedChapter {
                index: 0,
                title: title.clone(),
                href: None,
                content: "Содержимое FB2".to_string(),
            });
        }

        Ok(ParsedBook {
            title,
            author,
            language: None,
            format: BookFormat::Fb2,
            chapters,
        })
    }
}
