use std::fs;
use std::path::Path;

use ebook_core::{BookFormat, CoreError, CoreResult};
use roxmltree::{Document, Node};

use crate::{BookParser, ParsedBook, ParsedChapter};

pub struct Fb2Parser;

impl Fb2Parser {
    pub fn new() -> Self {
        Self
    }

    fn decode_xml_bytes(bytes: Vec<u8>) -> CoreResult<String> {
        if std::str::from_utf8(&bytes).is_ok() {
            let xml = unsafe { String::from_utf8_unchecked(bytes) };
            return Ok(xml);
        }

        let (cow, _, had_errors) = encoding_rs::WINDOWS_1251.decode(&bytes);
        if had_errors {
            return Err(CoreError::ParseError(
                "не удалось декодировать FB2-файл ни в UTF-8, ни в Windows-1251".into(),
            ));
        }

        Ok(cow.into_owned())
    }

    fn extract_metadata(doc: &Document) -> (String, String) {
        let mut title = "Без названия".to_string();
        let mut author = "Неизвестный автор".to_string();

        let Some(title_info) = doc.descendants().find(|n| n.has_tag_name("title-info")) else {
            return (title, author);
        };

        if let Some(book_title_node) = title_info.children().find(|n| n.has_tag_name("book-title"))
        {
            if let Some(text) = book_title_node.text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    title = trimmed.to_string();
                }
            }
        }

        if let Some(author_node) = title_info.children().find(|n| n.has_tag_name("author")) {
            let first = author_node
                .children()
                .find(|n| n.has_tag_name("first-name"))
                .and_then(|n| n.text())
                .map(str::trim)
                .unwrap_or("");
            let last = author_node
                .children()
                .find(|n| n.has_tag_name("last-name"))
                .and_then(|n| n.text())
                .map(str::trim)
                .unwrap_or("");

            let full = match (first.is_empty(), last.is_empty()) {
                (false, false) => format!("{first} {last}"),
                (false, true) => first.to_string(),
                (true, false) => last.to_string(),
                (true, true) => String::new(),
            };

            if !full.is_empty() {
                author = full;
            }
        }

        (title, author)
    }

    fn extract_section_title(section: Node<'_, '_>, fallback_index: usize) -> String {
        if let Some(title_node) = section.children().find(|n| n.has_tag_name("title")) {
            let mut title_buf = String::new();
            for p in title_node.descendants().filter(|n| n.has_tag_name("p")) {
                if let Some(txt) = p.text() {
                    let trimmed = txt.trim();
                    if !trimmed.is_empty() {
                        if !title_buf.is_empty() {
                            title_buf.push(' ');
                        }
                        title_buf.push_str(trimmed);
                    }
                }
            }
            if !title_buf.is_empty() {
                return title_buf;
            }
        }

        format!("Глава {}", fallback_index + 1)
    }

    fn collect_section_text(section: Node<'_, '_>) -> String {
        let mut content = String::with_capacity(4096);

        for p in section.descendants().filter(|n| n.has_tag_name("p")) {
            if let Some(txt) = p.text() {
                content.push_str(txt);
                content.push('\n');
            }
        }

        content
    }

    fn is_note_body(body: Node<'_, '_>) -> bool {
        body.attribute("name")
            .map(|name| {
                let lower = name.trim().to_ascii_lowercase();
                lower == "notes" || lower == "comments"
            })
            .unwrap_or(false)
    }

    fn is_numeric_note_title(title: &str) -> bool {
        let trimmed = title.trim();
        !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit())
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

        let xml_str = Self::decode_xml_bytes(bytes)?;

        let doc = Document::parse(&xml_str).map_err(|err| {
            CoreError::ParseError(format!("ошибка парсинга XML в FB2: {err}"))
        })?;

        let (title, author) = Self::extract_metadata(&doc);

        let mut chapters = Vec::new();
        let mut notes_buffer = String::with_capacity(8192);
        let mut index = 0;

        let root_children: Vec<Node<'_, '_>> = doc
            .root()
            .children()
            .find(|n| n.has_tag_name("FictionBook"))
            .map(|fb| fb.children().collect())
            .unwrap_or_else(|| doc.root().children().collect());

        for body in root_children.into_iter().filter(|n| n.has_tag_name("body")) {
            let note_body = Self::is_note_body(body);
            for section in body.children().filter(|n| n.has_tag_name("section")) {
                let content = Self::collect_section_text(section);
                if content.chars().any(|c| !c.is_whitespace()) {
                    let chapter_title = Self::extract_section_title(section, index);

                    if note_body || Self::is_numeric_note_title(&chapter_title) {
                        notes_buffer.push_str(&format!("[{chapter_title}]\n{}\n\n", content.trim()));
                    } else {
                        chapters.push(ParsedChapter {
                            index,
                            title: chapter_title,
                            href: None,
                            content,
                        });
                        index += 1;
                    }
                }
            }
        }

        if !notes_buffer.trim().is_empty() {
            chapters.push(ParsedChapter {
                index,
                title: "Примечания".to_string(),
                href: None,
                content: notes_buffer,
            });
        }

        if chapters.is_empty() {
            chapters.push(ParsedChapter {
                index: 0,
                title: title.clone(),
                href: None,
                content: String::new(),
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
