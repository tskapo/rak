// Store des préférences par dossier : view_mode (list/column/detail), sort_by, sort_dir.
// Lit depuis SQLite via tauri.ts au changement de dossier.
// Persiste automatiquement dans SQLite à chaque modification.

import { getFolderPrefs, saveFolderPrefs, type FolderPrefs } from '$lib/tauri'

// ── State ─────────────────────────────────────────────────────────────────────

const DEFAULT_PREFS: Omit<FolderPrefs, 'path'> = {
  viewMode: 'list',
  sortBy:   'name',
  sortDir:  'asc',
}

let prefs = $state<FolderPrefs>({ path: '', ...DEFAULT_PREFS })

// ── Public API ────────────────────────────────────────────────────────────────

/** Load preferences for `path` from SQLite (falls back to defaults if not stored). */
export async function loadFolderPrefs(path: string): Promise<void> {
  prefs = await getFolderPrefs(path)
}

/** The current folder preferences. */
export function getPrefs(): FolderPrefs {
  return prefs
}

/** Change the view mode and persist immediately. */
export async function setViewMode(viewMode: FolderPrefs['viewMode']): Promise<void> {
  prefs = { ...prefs, viewMode }
  await saveFolderPrefs(prefs).catch(console.error)
}

/** Change the sort column and persist immediately. */
export async function setSortBy(sortBy: FolderPrefs['sortBy']): Promise<void> {
  prefs = { ...prefs, sortBy }
  await saveFolderPrefs(prefs).catch(console.error)
}

/** Toggle or set the sort direction and persist immediately. */
export async function setSortDir(sortDir: FolderPrefs['sortDir']): Promise<void> {
  prefs = { ...prefs, sortDir }
  await saveFolderPrefs(prefs).catch(console.error)
}
