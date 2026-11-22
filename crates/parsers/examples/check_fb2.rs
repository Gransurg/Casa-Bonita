use std::env;
use std::path::Path;

use ebook_parsers::{BookParser, Fb2Parser};

fn main() {
    let Some(file_arg) = env::args().nth(1) else {
        eprintln!("Использование: cargo run -p ebook-parsers --example check_fb2 -- <путь/к/файлу.fb2>");
        std::process::exit(1);
    };

    let parser = Fb2Parser::new();
    let path = Path::new(&file_arg);

    match parser.parse(path) {
        Ok(book) => {
            println!("Формат:    {}", book.format);
            println!("Автор:     {}", book.author);
            println!("Название:  {}", book.title);
            println!("Глав:      {}", book.chapters.len());

            if let Some(first) = book.chapters.first() {
                let preview: String = first.content.chars().take(300).collect();
                println!("--- Первая глава ({}) ---\n{preview}...", first.title);
            }
        }
        Err(err) => {
            eprintln!("Ошибка разбора FB2: {err}");
            std::process::exit(1);
        }
    }
}
