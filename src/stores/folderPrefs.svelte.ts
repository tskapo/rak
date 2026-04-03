// Store des préférences par dossier : sort_by, sort_dir (par dossier) + view_mode (global).
// Le mode d'affichage est global : chargé une seule fois depuis app_prefs au démarrage,
// sauvegardé dans app_prefs à chaque changement. Il ne varie plus dossier par dossier.
// sort_by et sort_dir restent par dossier, lus/écrits dans folder_prefs.

import { getFolderPrefs, saveFolderPrefs, getAppPref, saveAppPref, type FolderPrefs } from '$lib/tauri'

// ── State ─────────────────────────────────────────────────────────────────────

let prefs = $state<FolderPrefs>({ path: '', viewMode: 'list', sortBy: 'name', sortDir: 'asc' })

// Load the global view mode once at startup.
if (typeof window !== 'undefined') {
  getAppPref('view_mode').then(v => {
    if (v === 'list' || v === 'column' || v === 'detail') {
      prefs = { ...prefs, viewMode: v }
    }
  }).catch(() => {})
}

// ── Public API ────────────────────────────────────────────────────────────────

/**
 * Load sort preferences for `path` from SQLite.
 * viewMode is NOT restored from here — it is global and already loaded at startup.
 */
export async function loadFolderPrefs(path: string): Promise<void> {
  const loaded = await getFolderPrefs(path)
  // Preserve the current global viewMode — only update sort prefs.
  prefs = { ...loaded, viewMode: prefs.viewMode }
}

/** The current folder preferences (including global viewMode). */
export function getPrefs(): FolderPrefs {
  return prefs
}

/** Change the view mode globally and persist to app_prefs. */
export async function setViewMode(viewMode: FolderPrefs['viewMode']): Promise<void> {
  prefs = { ...prefs, viewMode }
  await saveAppPref('view_mode', viewMode).catch(console.error)
}

/** Change the sort column and persist to folder_prefs. */
export async function setSortBy(sortBy: FolderPrefs['sortBy']): Promise<void> {
  prefs = { ...prefs, sortBy }
  await saveFolderPrefs(prefs).catch(console.error)
}

/** Toggle or set the sort direction and persist to folder_prefs. */
export async function setSortDir(sortDir: FolderPrefs['sortDir']): Promise<void> {
  prefs = { ...prefs, sortDir }
  await saveFolderPrefs(prefs).catch(console.error)
}
