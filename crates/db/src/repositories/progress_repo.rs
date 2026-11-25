use rusqlite::{params, Connection, Result};

pub struct ProgressRepository;

impl ProgressRepository {
    pub fn save_progress(
        conn: &Connection,
        book_id: i64,
        last_position: &str,
        percentage: f64,
        time_spent_delta: i64,
    ) -> Result<()> {
        conn.execute(
            r#"
            INSERT INTO reading_progress (book_id, last_position, percentage, last_read_at, total_time_spent)
            VALUES (?1, ?2, ?3, strftime('%s', 'now'), ?4)
            ON CONFLICT(book_id) DO UPDATE SET
                last_position = excluded.last_position,
                percentage = excluded.percentage,
                last_read_at = strftime('%s', 'now'),
                total_time_spent = total_time_spent + excluded.total_time_spent
            "#,
            params![book_id, last_position, percentage, time_spent_delta],
        )?;
        Ok(())
    }

    pub fn get_progress(conn: &Connection, book_id: i64) -> Result<Option<(String, f64, i64)>> {
        let mut stmt = conn.prepare(
            "SELECT last_position, percentage, total_time_spent FROM reading_progress WHERE book_id = ?1"
        )?;

        let mut rows = stmt.query(params![book_id])?;
        if let Some(row) = rows.next()? {
            let last_position: String = row.get(0)?;
            let percentage: f64 = row.get(1)?;
            let total_time_spent: i64 = row.get(2)?;
            Ok(Some((last_position, percentage, total_time_spent)))
        } else {
            Ok(None)
        }
    }
}
