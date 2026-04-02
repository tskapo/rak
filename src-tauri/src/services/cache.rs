// CacheService: SQLite access layer for Räk via sqlx.
// Uses runtime queries (no compile-time DATABASE_URL required).
// All database reads and writes go through this module.
// Commands and services never use sqlx directly.

use sqlx::{Row, SqlitePool};
use serde::{Deserialize, Serialize};

// ── Shared types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path:        String,
    pub name:        String,
    pub is_dir:      bool,
    pub size:        Option<i64>,
    pub modified_at: Option<i64>,
    pub mime_type:   Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderPrefs {
    pub path:      String,
    pub view_mode: String,
    pub sort_by:   String,
    pub sort_dir:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPrefs {
    pub theme:            String,
    pub locale:           String,
    pub ui_scale:         f32,
    /// Watch directories recursively (true) or only the immediate dir (false).
    /// Defaults to false — safer on large NAS trees.
    pub watch_recursive:  bool,
    /// True on first launch — the frontend shows a language picker before the app.
    /// Set to false after the user selects a locale.
    pub first_launch:     bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskLabel {
    pub path:        String,
    pub color:       String,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id:    i64,
    pub name:  String,
    pub color: String,
}

// ── Database initialisation ──────────────────────────────────────────────────

/// Open (or create) the SQLite database and apply the schema.
pub async fn open(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", db_path)).await?;

    sqlx::query(include_str!("../db/schema.sql"))
        .execute(&pool)
        .await?;

    Ok(pool)
}

// ── file_cache ────────────────────────────────────────────────────────────────

/// Return all cached entries whose direct parent is `dir_path`.
pub async fn list_dir(pool: &SqlitePool, dir_path: &str) -> Result<Vec<FileEntry>, sqlx::Error> {
    let pattern = format!("{}/%", dir_path);
    let exclude = format!("{}/%/%", dir_path);

    let rows = sqlx::query(
        "SELECT path, name, is_dir, size, modified_at, mime_type
         FROM   file_cache
         WHERE  path LIKE ? AND path NOT LIKE ?
         ORDER  BY is_dir DESC, name ASC",
    )
    .bind(&pattern)
    .bind(&exclude)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| FileEntry {
            path:        r.get("path"),
            name:        r.get("name"),
            is_dir:      r.get("is_dir"),
            size:        r.get("size"),
            modified_at: r.get("modified_at"),
            mime_type:   r.get("mime_type"),
        })
        .collect())
}

/// Insert or update a single file entry in the cache.
pub async fn upsert_file(pool: &SqlitePool, entry: &FileEntry, indexed_at: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO file_cache (path, name, is_dir, size, modified_at, mime_type, indexed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(path) DO UPDATE SET
             name        = excluded.name,
             is_dir      = excluded.is_dir,
             size        = excluded.size,
             modified_at = excluded.modified_at,
             mime_type   = excluded.mime_type,
             indexed_at  = excluded.indexed_at",
    )
    .bind(&entry.path)
    .bind(&entry.name)
    .bind(entry.is_dir)
    .bind(entry.size)
    .bind(entry.modified_at)
    .bind(&entry.mime_type)
    .bind(indexed_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Return a single cached entry by exact path, or None if not present.
pub async fn get_file(pool: &SqlitePool, path: &str) -> Result<Option<FileEntry>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT path, name, is_dir, size, modified_at, mime_type
         FROM   file_cache
         WHERE  path = ?",
    )
    .bind(path)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| FileEntry {
        path:        r.get("path"),
        name:        r.get("name"),
        is_dir:      r.get("is_dir"),
        size:        r.get("size"),
        modified_at: r.get("modified_at"),
        mime_type:   r.get("mime_type"),
    }))
}

/// Return the most recent `indexed_at` timestamp among direct children of `dir_path`.
/// Returns None if the directory has no cached entries.
pub async fn get_dir_indexed_at(pool: &SqlitePool, dir_path: &str) -> Result<Option<i64>, sqlx::Error> {
    let pattern = format!("{}/%", dir_path);
    let exclude  = format!("{}/%/%", dir_path);

    let row = sqlx::query(
        "SELECT MAX(indexed_at) AS ts FROM file_cache WHERE path LIKE ? AND path NOT LIKE ?",
    )
    .bind(&pattern)
    .bind(&exclude)
    .fetch_one(pool)
    .await?;

    Ok(row.get("ts"))
}

/// Remove a file entry from the cache (e.g. after deletion detected by watcher).
pub async fn remove_file(pool: &SqlitePool, path: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM file_cache WHERE path = ?")
        .bind(path)
        .execute(pool)
        .await?;

    Ok(())
}

/// Remove all cached entries whose path starts with `dir_path/`.
/// Used when a directory is deleted or renamed — cascades the removal to all children.
pub async fn remove_children(pool: &SqlitePool, dir_path: &str) -> Result<(), sqlx::Error> {
    let prefix = format!("{}/", dir_path);

    sqlx::query("DELETE FROM file_cache WHERE path LIKE ?")
        .bind(format!("{}%", prefix))
        .execute(pool)
        .await?;

    Ok(())
}

// ── folder_prefs ──────────────────────────────────────────────────────────────

/// Return preferences for a folder, or sensible defaults if none are stored.
pub async fn get_folder_prefs(pool: &SqlitePool, path: &str) -> Result<FolderPrefs, sqlx::Error> {
    let row = sqlx::query(
        "SELECT path, view_mode, sort_by, sort_dir FROM folder_prefs WHERE path = ?",
    )
    .bind(path)
    .fetch_optional(pool)
    .await?;

    Ok(row
        .map(|r| FolderPrefs {
            path:      r.get("path"),
            view_mode: r.get("view_mode"),
            sort_by:   r.get("sort_by"),
            sort_dir:  r.get("sort_dir"),
        })
        .unwrap_or_else(|| FolderPrefs {
            path:      path.to_string(),
            view_mode: "list".to_string(),
            sort_by:   "name".to_string(),
            sort_dir:  "asc".to_string(),
        }))
}

/// Persist folder preferences immediately.
pub async fn save_folder_prefs(pool: &SqlitePool, prefs: &FolderPrefs) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO folder_prefs (path, view_mode, sort_by, sort_dir, updated_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(path) DO UPDATE SET
             view_mode  = excluded.view_mode,
             sort_by    = excluded.sort_by,
             sort_dir   = excluded.sort_dir,
             updated_at = excluded.updated_at",
    )
    .bind(&prefs.path)
    .bind(&prefs.view_mode)
    .bind(&prefs.sort_by)
    .bind(&prefs.sort_dir)
    .bind(now_unix())
    .execute(pool)
    .await?;

    Ok(())
}

// ── recents ───────────────────────────────────────────────────────────────────

/// Return the most recently visited entries, newest first.
pub async fn get_recents(pool: &SqlitePool, limit: i64) -> Result<Vec<FileEntry>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT path, name, is_dir, visited_at FROM recents ORDER BY visited_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| FileEntry {
            path:        r.get("path"),
            name:        r.get("name"),
            is_dir:      r.get("is_dir"),
            size:        None,
            modified_at: r.get("visited_at"),
            mime_type:   None,
        })
        .collect())
}

/// Record a visit — upserts the entry and prunes to 50 rows (FIFO).
pub async fn add_recent(pool: &SqlitePool, entry: &FileEntry) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO recents (path, name, is_dir, visited_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(path) DO UPDATE SET visited_at = excluded.visited_at",
    )
    .bind(&entry.path)
    .bind(&entry.name)
    .bind(entry.is_dir)
    .bind(now_unix())
    .execute(pool)
    .await?;

    sqlx::query(
        "DELETE FROM recents WHERE path NOT IN
         (SELECT path FROM recents ORDER BY visited_at DESC LIMIT 50)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ── app_prefs ─────────────────────────────────────────────────────────────────

/// Return all global preferences with defaults applied.
pub async fn get_app_prefs(pool: &SqlitePool) -> Result<AppPrefs, sqlx::Error> {
    let rows = sqlx::query("SELECT key, value FROM app_prefs")
        .fetch_all(pool)
        .await?;

    let mut theme           = "system".to_string();
    let mut locale          = "en".to_string();
    let mut ui_scale        = 1.0_f32;
    let mut watch_recursive = false;
    let mut first_launch    = true;

    for row in rows {
        let key: String = row.get("key");
        let val: String = row.get("value");
        match key.as_str() {
            "theme"           => theme           = val,
            "locale"          => locale          = val,
            "ui_scale"        => ui_scale        = val.parse().unwrap_or(1.0),
            "watch_recursive" => watch_recursive = val == "true",
            "first_launch"    => first_launch    = val != "false",
            _                 => {}
        }
    }

    Ok(AppPrefs { theme, locale, ui_scale, watch_recursive, first_launch })
}

/// Persist a single app preference key/value pair.
pub async fn save_app_pref(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO app_prefs (key, value, updated_at)
         VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .bind(now_unix())
    .execute(pool)
    .await?;

    Ok(())
}

// ── disk_labels ───────────────────────────────────────────────────────────────

/// Return the label for a mounted volume, if one has been set.
pub async fn get_disk_label(pool: &SqlitePool, path: &str) -> Result<Option<DiskLabel>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT path, color, custom_name FROM disk_labels WHERE path = ?",
    )
    .bind(path)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| DiskLabel {
        path:        r.get("path"),
        color:       r.get("color"),
        custom_name: r.get("custom_name"),
    }))
}

/// Persist (or update) the label for a mounted volume.
pub async fn save_disk_label(pool: &SqlitePool, label: &DiskLabel) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO disk_labels (path, color, custom_name, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(path) DO UPDATE SET
             color       = excluded.color,
             custom_name = excluded.custom_name,
             updated_at  = excluded.updated_at",
    )
    .bind(&label.path)
    .bind(&label.color)
    .bind(&label.custom_name)
    .bind(now_unix())
    .execute(pool)
    .await?;

    Ok(())
}

// ── tags ──────────────────────────────────────────────────────────────────────

/// Return all user-defined tags.
pub async fn get_tags(pool: &SqlitePool) -> Result<Vec<Tag>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, name, color FROM tags ORDER BY name ASC")
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| Tag { id: r.get("id"), name: r.get("name"), color: r.get("color") })
        .collect())
}

/// Create a new tag and return it with its generated id.
pub async fn create_tag(pool: &SqlitePool, name: &str, color: &str) -> Result<Tag, sqlx::Error> {
    let id = sqlx::query("INSERT INTO tags (name, color) VALUES (?, ?)")
        .bind(name)
        .bind(color)
        .execute(pool)
        .await?
        .last_insert_rowid();

    Ok(Tag { id, name: name.to_string(), color: color.to_string() })
}

/// Delete a tag — cascades automatically to file_tags.
pub async fn delete_tag(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Return all tags assigned to a file or folder.
pub async fn get_file_tags(pool: &SqlitePool, file_path: &str) -> Result<Vec<Tag>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT t.id, t.name, t.color
         FROM   tags t
         JOIN   file_tags ft ON ft.tag_id = t.id
         WHERE  ft.file_path = ?
         ORDER  BY t.name ASC",
    )
    .bind(file_path)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Tag { id: r.get("id"), name: r.get("name"), color: r.get("color") })
        .collect())
}

/// Replace the full tag set for a file (delete all then insert).
pub async fn set_file_tags(pool: &SqlitePool, file_path: &str, tag_ids: &[i64]) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM file_tags WHERE file_path = ?")
        .bind(file_path)
        .execute(pool)
        .await?;

    for &tag_id in tag_ids {
        sqlx::query("INSERT INTO file_tags (file_path, tag_id) VALUES (?, ?)")
            .bind(file_path)
            .bind(tag_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
