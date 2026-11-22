use std::fs;
use std::path::Path;

use ebook_core::{BookFormat, CoreError, CoreResult};

use crate::{BookParser, ParsedBook, ParsedChapter};

pub struct TxtParser;

impl TxtParser {
    pub fn new() -> Self {
        Self
    }

    fn extract_title(path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Без названия")
            .to_string()
    }
}

impl Default for TxtParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BookParser for TxtParser {
    fn format(&self) -> BookFormat {
        BookFormat::Txt
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
    }

    fn parse(&self, path: &Path) -> CoreResult<ParsedBook> {
        if !self.can_parse(path) {
            return Err(CoreError::ParseError(format!(
                "файл '{}' не имеет расширения .txt",
                path.display()
            )));
        }

        // Читаем файл в сырые байты
        let bytes = fs::read(path).map_err(|err| {
            CoreError::ParseError(format!(
                "не удалось прочитать TXT-файл '{}': {err}",
                path.display()
            ))
        })?;

        // Пробуем распарсить как UTF-8, если не выходит — декодируем из Windows-1251
        let content = match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(_) => {
                let (cow, _, _) = encoding_rs::WINDOWS_1251.decode(&bytes);
                cow.into_owned()
            }
        };

        let title = Self::extract_title(path);

        let chapters = if content.trim().is_empty() {
            Vec::new()
        } else {
            vec![ParsedChapter {
                index: 0,
                title: title.clone(),
                href: None,
                content,
            }]
        };

        Ok(ParsedBook {
            title,
            author: "Неизвестный автор".to_string(),
            language: None,
            format: BookFormat::Txt,
            chapters,
        })
    }
}
