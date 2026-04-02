-- SQLite schema for Räk.
-- Applied at startup via sqlx migrate — each statement is idempotent.

-- File metadata cache populated by IndexerService.
CREATE TABLE IF NOT EXISTS file_cache (
    path        TEXT    PRIMARY KEY,
    name        TEXT    NOT NULL,
    is_dir      BOOLEAN NOT NULL,
    size        INTEGER,
    modified_at INTEGER,
    mime_type   TEXT,
    indexed_at  INTEGER NOT NULL
);

-- Per-folder display preferences (view mode, sort).
CREATE TABLE IF NOT EXISTS folder_prefs (
    path       TEXT PRIMARY KEY,
    view_mode  TEXT NOT NULL DEFAULT 'list',  -- 'list' | 'column' | 'detail'
    sort_by    TEXT NOT NULL DEFAULT 'name',  -- 'name' | 'date' | 'size'
    sort_dir   TEXT NOT NULL DEFAULT 'asc',   -- 'asc' | 'desc'
    updated_at INTEGER NOT NULL
);

-- Navigation history — capped at 50 entries (FIFO enforced in code).
CREATE TABLE IF NOT EXISTS recents (
    path       TEXT    PRIMARY KEY,
    name       TEXT    NOT NULL,
    is_dir     BOOLEAN NOT NULL,
    visited_at INTEGER NOT NULL
);

-- Global key/value app preferences.
-- Reserved keys: 'theme' ('light'|'dark'|'system'), 'locale' ('fr'|'en'|'ka'), 'ui_scale' (float string).
CREATE TABLE IF NOT EXISTS app_prefs (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Custom color and display name per mounted volume or network share.
CREATE TABLE IF NOT EXISTS disk_labels (
    path        TEXT PRIMARY KEY,  -- mount path, e.g. /Volumes/NAS
    color       TEXT NOT NULL,     -- hex color, e.g. '#4a90d9'
    custom_name TEXT,              -- optional display name, e.g. 'Home NAS'
    updated_at  INTEGER NOT NULL
);

-- User-defined tags (free color palette).
CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT    NOT NULL UNIQUE,
    color TEXT    NOT NULL  -- hex color
);

-- Many-to-many association between file/folder paths and tags.
CREATE TABLE IF NOT EXISTS file_tags (
    file_path TEXT    NOT NULL,
    tag_id    INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (file_path, tag_id)
);
