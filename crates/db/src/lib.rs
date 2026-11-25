use rusqlite::Connection;
use std::path::Path;

pub mod repositories;
pub use repositories::BookRepository;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("../../migrations");
}

pub fn init_database(db_path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let mut conn = Connection::open(db_path)?;
    
    embedded::migrations::runner().run(&mut conn)?;
    
    Ok(conn)
}
