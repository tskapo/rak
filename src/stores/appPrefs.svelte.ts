// Store des préférences globales de l'application : thème, langue, échelle UI, etc.
// Charge depuis SQLite via get_app_prefs au démarrage — avant le premier rendu.
// Persiste chaque changement via save_app_pref.
// Notifie i18n (initLocale) au chargement initial.

import { getAppPrefs, saveAppPref, type AppPrefs } from '$lib/tauri'
import { initLocale, type Locale } from '$lib/i18n.svelte'

// ── State ─────────────────────────────────────────────────────────────────────

let prefs = $state<AppPrefs | null>(null)

// ── Public API ────────────────────────────────────────────────────────────────

/** Load preferences from SQLite. Must be called once at app startup. */
export async function loadAppPrefs(): Promise<void> {
  prefs = await getAppPrefs()
  initLocale(prefs.locale as Locale)
}

/** True until the user has picked a language on first launch. */
export function isFirstLaunch(): boolean {
  return prefs?.firstLaunch ?? true
}

/** Mark first launch as complete and persist the flag. */
export async function completeFirstLaunch(): Promise<void> {
  await saveAppPref('first_launch', 'false')
  if (prefs) prefs = { ...prefs, firstLaunch: false }
}

/** The loaded preferences, or null before loadAppPrefs() resolves. */
export function getPrefs(): AppPrefs | null {
  return prefs
}
