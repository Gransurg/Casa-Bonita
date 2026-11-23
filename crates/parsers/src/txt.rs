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

    fn decode_bytes(bytes: Vec<u8>) -> CoreResult<String> {
        if std::str::from_utf8(&bytes).is_ok() {
            // Безопасно, так как валидность UTF-8 уже проверена выше.
            let text = unsafe { String::from_utf8_unchecked(bytes) };
            return Ok(text);
        }

        let (cow, _, had_errors) = encoding_rs::WINDOWS_1251.decode(&bytes);
        if had_errors {
            return Err(CoreError::ParseError(
                "не удалось декодировать TXT-файл ни в UTF-8, ни в Windows-1251".into(),
            ));
        }

        Ok(cow.into_owned())
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

        let bytes = fs::read(path).map_err(|err| {
            CoreError::ParseError(format!(
                "не удалось прочитать TXT-файл '{}': {err}",
                path.display()
            ))
        })?;

        let content = Self::decode_bytes(bytes)?;
        let title = Self::extract_title(path);

        let has_text = content.chars().any(|c| !c.is_whitespace());
        let chapters = if !has_text {
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
