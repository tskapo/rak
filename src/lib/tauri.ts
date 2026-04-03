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

export interface AppPrefs {
  theme:          'light' | 'dark' | 'system'
  locale:         'fr' | 'en' | 'ka'
  uiScale:        number
  watchRecursive: boolean
  /** True on first launch — frontend shows the language picker before the app. */
  firstLaunch:    boolean
}

export interface DiskLabel {
  path:       string
  color:      string
  customName?: string
}

export interface Tag {
  id:    number
  name:  string
  color: string
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
 * Return the mounted volumes visible to the user (macOS: reads /Volumes).
 *
 * @returns Array of FileEntry with isDir=true, one per volume.
 */
export async function listVolumes(): Promise<FileEntry[]> {
  return invoke('list_volumes')
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

/**
 * Open `path` with the system's default application.
 *
 * @throws string — if the OS command fails to spawn
 */
export async function openPath(path: string): Promise<void> {
  return invoke('open_path', { path })
}

/**
 * Permanently delete the file or directory at `path`.
 *
 * Directories are deleted recursively. The entry is also removed from the
 * SQLite cache so the UI reflects the deletion on the next `listDir` call.
 *
 * @throws string — if the file cannot be deleted or does not exist
 */
export async function deleteEntry(path: string): Promise<void> {
  return invoke('delete_entry', { path })
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

// ── App preferences commands ──────────────────────────────────────────────────

/**
 * Return all global application preferences with defaults applied.
 *
 * @throws string — if the database query fails
 */
export async function getAppPrefs(): Promise<AppPrefs> {
  return invoke('get_app_prefs')
}

/**
 * Persist a single global preference by key.
 *
 * Keys: `"theme"` | `"locale"` | `"ui_scale"` | `"watch_recursive"`
 *
 * @throws string — if the database write fails
 */
export async function getAppPref(key: string): Promise<string | null> {
  return invoke('get_app_pref', { key })
}

export async function saveAppPref(key: string, value: string): Promise<void> {
  return invoke('save_app_pref', { key, value })
}

// ── Tag commands ──────────────────────────────────────────────────────────────

/**
 * Return all user-defined tags, sorted alphabetically.
 *
 * @throws string — if the database query fails
 */
export async function getTags(): Promise<Tag[]> {
  return invoke('get_tags')
}

/**
 * Create a new tag with the given name and hex color.
 *
 * @returns the created tag including its auto-generated `id`
 * @throws string — if the name already exists (UNIQUE constraint)
 */
export async function createTag(name: string, color: string): Promise<Tag> {
  return invoke('create_tag', { name, color })
}

/**
 * Update the name and color of an existing tag.
 *
 * @throws string — if the name already exists (UNIQUE constraint)
 */
export async function updateTag(id: number, name: string, color: string): Promise<void> {
  return invoke('update_tag', { id, name, color })
}

/**
 * Delete the tag with the given `id`.
 *
 * Cascades automatically to all file–tag associations.
 *
 * @throws string — if the database write fails
 */
export async function deleteTag(id: number): Promise<void> {
  return invoke('delete_tag', { id })
}

/**
 * Return all tags assigned to the file or folder at `path`.
 *
 * @throws string — if the database query fails
 */
export async function getFileTags(path: string): Promise<Tag[]> {
  return invoke('get_file_tags', { path })
}

/**
 * Replace the full tag set for the file at `path` with `tagIds`.
 *
 * Pass an empty array to remove all tags.
 *
 * @throws string — if the database write fails
 */
export async function setFileTags(path: string, tagIds: number[]): Promise<void> {
  return invoke('set_file_tags', { path, tagIds })
}

// ── Disk label commands ───────────────────────────────────────────────────────

/**
 * Return the label for the mounted volume at `path`, or `null` if none is set.
 *
 * @throws string — if the database query fails
 */
export async function getDiskLabel(path: string): Promise<DiskLabel | null> {
  return invoke('get_disk_label', { path })
}

/**
 * Persist (or update) the label for the mounted volume described by `label`.
 *
 * @throws string — if the database write fails
 */
export async function saveDiskLabel(label: DiskLabel): Promise<void> {
  return invoke('save_disk_label', { label })
}
