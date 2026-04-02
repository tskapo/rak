// Wrappers typés autour de invoke() Tauri.
// Toutes les commandes IPC passent par ce fichier — jamais d'invoke() direct dans les composants.
// Chaque fonction documente ses paramètres, son type de retour et les erreurs possibles.

import { invoke } from '@tauri-apps/api/core'

// ── Shared types ──────────────────────────────────────────────────────────────

export interface FileEntry {
  path:       string
  name:       string
  isDir:      boolean
  size?:      number
  modifiedAt?: number
  mimeType?:  string
}

export interface FolderPrefs {
  path:     string
  viewMode: 'list' | 'column' | 'detail'
  sortBy:   'name' | 'date' | 'size'
  sortDir:  'asc' | 'desc'
}

/** Returned by `listDir` — entries plus cache metadata. */
export interface DirResult {
  entries:   FileEntry[]
  /** Unix timestamp (seconds) of the last index run. Null if never indexed. */
  indexedAt: number | null
}

// ── Filesystem commands ───────────────────────────────────────────────────────

/**
 * List the direct children of `path` from the SQLite cache.
 *
 * Returns immediately with cached data. If the cache is empty or older than
 * 5 minutes, a background re-scan is triggered automatically. Listen to the
 * `indexer:progress` event to update the UI as new entries arrive.
 *
 * @throws string — if the cache query fails
 */
export async function listDir(path: string): Promise<DirResult> {
  return invoke('list_dir', { path })
}

/**
 * Return metadata for a single file or directory.
 *
 * Tries the cache first; falls back to a live filesystem stat if not cached.
 *
 * @throws string — if the path cannot be accessed
 */
export async function getMetadata(path: string): Promise<FileEntry> {
  return invoke('get_metadata', { path })
}

/**
 * Read up to `size` bytes from `path` starting at `offset`.
 *
 * Returns a `number[]` representing raw bytes. Convert with:
 * `const bytes = new Uint8Array(await readFileChunk(path, 0, 4096))`
 *
 * @throws string — if the file cannot be opened or read
 */
export async function readFileChunk(
  path:   string,
  offset: number,
  size:   number,
): Promise<number[]> {
  return invoke('read_file_chunk', { path, offset, size })
}

/**
 * Trigger an immediate background re-scan of `path`.
 *
 * Call this when the user clicks the ↺ rescan button. Progress is reported
 * via `indexer:progress` events.
 *
 * @throws string — (unlikely) if the spawn fails
 */
export async function rescanDir(path: string): Promise<void> {
  return invoke('rescan_dir', { path })
}

// ── Folder preferences commands ───────────────────────────────────────────────

/**
 * Return the display preferences for `path`.
 *
 * Returns defaults (list / name / asc) if no preferences are stored yet.
 *
 * @throws string — if the database query fails
 */
export async function getFolderPrefs(path: string): Promise<FolderPrefs> {
  return invoke('get_folder_prefs', { path })
}

/**
 * Persist the display preferences for `prefs.path`.
 *
 * Call on every view/sort change so preferences survive between sessions.
 *
 * @throws string — if the database write fails
 */
export async function saveFolderPrefs(prefs: FolderPrefs): Promise<void> {
  return invoke('save_folder_prefs', { prefs })
}

// ── Recents commands ──────────────────────────────────────────────────────────

/**
 * Return the `limit` most recently visited entries, newest first.
 *
 * The `modifiedAt` field on each entry contains the `visited_at` timestamp
 * (when the user last opened it), not the file's own modification date.
 *
 * @throws string — if the database query fails
 */
export async function getRecents(limit: number): Promise<FileEntry[]> {
  return invoke('get_recents', { limit })
}

/**
 * Record a visit to `entry`.
 *
 * Upserts into `recents` and prunes the table to 50 rows (FIFO).
 *
 * @throws string — if the database write fails
 */
export async function addRecent(entry: FileEntry): Promise<void> {
  return invoke('add_recent', { entry })
}
