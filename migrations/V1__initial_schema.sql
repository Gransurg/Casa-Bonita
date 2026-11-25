
/*
Model: SQLite 3.7
Database: SQLite 3.7
*/


-- Create tables section -------------------------------------------------

-- Table Users

CREATE TABLE Users
(
  id INTEGER NOT NULL,
  username TEXT NOT NULL,
  password_hash TEXT NOT NULL,
  created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT PK_Users PRIMARY KEY (id),
  CONSTRAINT username UNIQUE (username)
);

-- Table books

CREATE TABLE books
(
  id INTEGER NOT NULL,
  user_id INTEGER,
  title TEXT NOT NULL,
  author TEXT,
  file_path TEXT NOT NULL,
  format TEXT NOT NULL
        CONSTRAINT CheckFormat CHECK (UPPER(format) IN ('EPUB', 'PDF', 'TXT', 'FB2')),
  cover_image_path TEXT,
  is_favorite INTEGER NOT NULL DEFAULT 0,
  is_finished INTEGER NOT NULL DEFAULT 0,
  added_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT PK_books PRIMARY KEY (id),
  CONSTRAINT file_path UNIQUE (file_path),
  CONSTRAINT FK_books_user FOREIGN KEY (user_id) REFERENCES Users (id) ON DELETE CASCADE
);

CREATE INDEX IX_Relationship3 ON books (user_id);

-- Table annotations

CREATE TABLE annotations
(
  id INTEGER NOT NULL,
  book_id INTEGER,
  type TEXT NOT NULL
        CONSTRAINT CheckConstraintA1 CHECK (type IN ('highlight', 'note', 'bookmark')),
  text_content TEXT,
  note_content TEXT,
  created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  chapter_index INTEGER NOT NULL DEFAULT 0,
  page_index INTEGER NOT NULL DEFAULT 0,
  CONSTRAINT PK_annotations PRIMARY KEY (id),
  CONSTRAINT FK_annotations_book FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE
);

CREATE INDEX IX_Relationship5 ON annotations (book_id);

-- Table reading_progress

CREATE TABLE reading_progress
(
  id INTEGER NOT NULL,
  book_id INTEGER NOT NULL,
  last_position TEXT NOT NULL,
  percentage REAL NOT NULL
        CONSTRAINT CheckConstraintA1 CHECK (percentage BETWEEN 0 AND 100),
  last_read_at datetime NOT NULL,
  total_time_spent INTEGER NOT NULL DEFAULT 0,
  CONSTRAINT PK_reading_progress PRIMARY KEY (id),
  CONSTRAINT UQ_book_id UNIQUE (book_id),
  CONSTRAINT FK_progress_book FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE
);

CREATE INDEX IX_Relationship4 ON reading_progress (book_id);

