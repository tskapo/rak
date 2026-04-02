// Store du contenu du dossier courant : liste des FileEntry réactive.
// Se recharge via listDir() à chaque changement de dossier.
// Écoute les événements indexer:progress et watcher:change pour les mises à jour en temps réel.
// Expose l'état de chargement, l'heure du dernier index et les éventuelles erreurs.

import { listen } from '@tauri-apps/api/event'
import { listDir, type FileEntry } from '$lib/tauri'

interface WatcherChange { kind: string; path: string; newPath?: string }
interface IndexProgress { path: string; filesIndexed: number }

// ── State ─────────────────────────────────────────────────────────────────────

let entries    = $state<FileEntry[]>([])
let indexedAt  = $state<number | null>(null)
let loading    = $state(false)
let error      = $state<string | null>(null)
let activePath = ''

// ── Reactive event listeners ──────────────────────────────────────────────────

// These run once when the module is first imported (browser-side only).
if (typeof window !== 'undefined') {
  listen<WatcherChange>('watcher:change', () => {
    if (activePath) loadDir(activePath).catch(console.error)
  })

  listen<IndexProgress>('indexer:progress', ({ payload }) => {
    // Reload when the currently displayed directory received new entries
    if (activePath && payload.path === activePath) {
      loadDir(activePath).catch(console.error)
    }
  })
}

// ── Public API ────────────────────────────────────────────────────────────────

/** Load the contents of `path` from the SQLite cache. */
export async function loadDir(path: string): Promise<void> {
  activePath = path
  loading    = true
  error      = null

  try {
    const result = await listDir(path)
    entries   = result.entries
    indexedAt = result.indexedAt
  } catch (e) {
    error = String(e)
  } finally {
    loading = false
  }
}

/** The file entries for the current directory. */
export function getEntries(): FileEntry[] { return entries }

/** Unix timestamp (seconds) of the last index run, or null if never indexed. */
export function getIndexedAt(): number | null { return indexedAt }

/** True while a `list_dir` call is in flight. */
export function isLoading(): boolean { return loading }

/** The last error string, or null if the last load succeeded. */
export function getError(): string | null { return error }
