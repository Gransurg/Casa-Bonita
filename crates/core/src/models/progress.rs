use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, PartialEq)]
pub struct ReadingProgress {
    pub book_id: String,
    pub chapter_index: usize,
    pub current_page: u32,
    pub total_pages: u32,
    /// Доля прочитанного от всей книги в диапазоне `0.0..=1.0`.
    pub percentage: f64,
    pub minutes_left_in_chapter: Option<u32>,
    pub updated_at: i64,
}

impl ReadingProgress {
    pub fn new(
        book_id: impl Into<String>,
        chapter_index: usize,
        current_page: u32,
        total_pages: u32,
        percentage: f64,
        updated_at: i64,
    ) -> CoreResult<Self> {
        if total_pages > 0 && current_page > total_pages {
            return Err(CoreError::InvalidProgress(format!(
                "текущая страница ({current_page}) больше общего числа страниц ({total_pages})"
            )));
        }

        if !(0.0..=1.0).contains(&percentage) || percentage.is_nan() {
            return Err(CoreError::InvalidProgress(format!(
                "процент чтения ({percentage}) должен быть в диапазоне 0.0..=1.0"
            )));
        }

        Ok(Self {
            book_id: book_id.into(),
            chapter_index,
            current_page,
            total_pages,
            percentage,
            minutes_left_in_chapter: None,
            updated_at,
        })
    }

    pub fn percentage_percent(&self) -> u8 {
        (self.percentage * 100.0).round().clamp(0.0, 100.0) as u8
    }
}
