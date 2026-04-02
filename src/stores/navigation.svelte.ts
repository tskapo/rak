// Store de navigation : gère le dossier courant, l'historique (back/forward) et le breadcrumb.
// navigate(path) : change de dossier, sauvegarde les prefs de l'ancien dossier avant de partir.
// back() / forward() : navigation dans l'historique de session.

import { addRecent, type FileEntry } from '$lib/tauri'

// ── State ─────────────────────────────────────────────────────────────────────

let currentPath = $state<string>('')
let history     = $state<string[]>([])
let histIdx     = $state<number>(-1)

// ── Public API ────────────────────────────────────────────────────────────────

/** The currently displayed directory path. */
export function getCurrentPath(): string {
  return currentPath
}

/** True when there is a previous entry in the session history. */
export function canGoBack(): boolean {
  return histIdx > 0
}

/** True when there is a forward entry in the session history. */
export function canGoForward(): boolean {
  return histIdx < history.length - 1
}

/**
 * Navigate to `path`.
 * Appends to the history stack and records the visit in the `recents` table.
 */
export async function navigate(path: string): Promise<void> {
  if (path === currentPath) return

  // Discard any forward history when navigating to a new path
  history = [...history.slice(0, histIdx + 1), path]
  histIdx = history.length - 1
  currentPath = path

  const name = path.split('/').filter(Boolean).at(-1) ?? path
  const entry: FileEntry = { path, name, isDir: true }
  await addRecent(entry).catch(console.error)
}

/** Navigate to the previous directory in session history. */
export function back(): void {
  if (!canGoBack()) return
  histIdx    -= 1
  currentPath = history[histIdx]
}

/** Navigate forward after a `back()`. */
export function forward(): void {
  if (!canGoForward()) return
  histIdx    += 1
  currentPath = history[histIdx]
}

/**
 * Split the current path into breadcrumb segments.
 *
 * @returns Array of `{ label, path }` objects, root-to-leaf order.
 *
 * @example
 *   '/Volumes/NAS/Projets' → [
 *     { label: 'Volumes',  path: '/Volumes' },
 *     { label: 'NAS',      path: '/Volumes/NAS' },
 *     { label: 'Projets',  path: '/Volumes/NAS/Projets' },
 *   ]
 */
export function breadcrumbs(): { label: string; path: string }[] {
  if (!currentPath) return []

  const parts = currentPath.split('/').filter(Boolean)
  return parts.map((label, i) => ({
    label,
    path: '/' + parts.slice(0, i + 1).join('/'),
  }))
}
