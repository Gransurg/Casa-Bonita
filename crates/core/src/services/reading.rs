use crate::error::{CoreError, CoreResult};
use crate::models::ReadingProgress;

pub struct ReadingService;

impl ReadingService {
    /// Перелистывает на `delta` страниц (положительное число — вперёд, отрицательное — назад)
    /// и пересчитывает прогресс.
    pub fn step_page(
        progress: &ReadingProgress,
        delta: i32,
        updated_at: i64,
    ) -> CoreResult<ReadingProgress> {
        if progress.total_pages == 0 {
            return Err(CoreError::InvalidProgress(
                "невозможно перелистнуть страницу при нулевом общем числе страниц".into(),
            ));
        }

        let current = progress.current_page as i64 + delta as i64;
        let clamped_page = current.clamp(1, progress.total_pages as i64) as u32;
        let percentage = Self::calculate_percentage(clamped_page, progress.total_pages)?;

        let mut updated = ReadingProgress::new(
            progress.book_id.clone(),
            progress.chapter_index,
            clamped_page,
            progress.total_pages,
            percentage,
            updated_at,
        )?;
        updated.minutes_left_in_chapter = progress.minutes_left_in_chapter;

        Ok(updated)
    }

    /// Вычисляет долю прочитанного в диапазоне `0.0..=1.0` по текущей и общей страницам.
    pub fn calculate_percentage(current_page: u32, total_pages: u32) -> CoreResult<f64> {
        if total_pages == 0 {
            return Err(CoreError::InvalidProgress(
                "общее число страниц должно быть больше нуля".into(),
            ));
        }
        if current_page > total_pages {
            return Err(CoreError::InvalidProgress(format!(
                "текущая страница ({current_page}) больше общего числа страниц ({total_pages})"
            )));
        }

        let ratio = current_page as f64 / total_pages as f64;
        Ok(ratio.clamp(0.0, 1.0))
    }

    /// Оценивает примерное число минут до конца главы.
    pub fn estimate_minutes_left(pages_left: u32, seconds_per_page: u32) -> Option<u32> {
        if pages_left == 0 || seconds_per_page == 0 {
            return Some(0);
        }

        let total_seconds = pages_left.saturating_mul(seconds_per_page);
        let minutes = (total_seconds + 59) / 60; // округление вверх до целых минут
        Some(minutes.max(1))
    }
}
