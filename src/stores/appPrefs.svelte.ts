// Store des préférences globales de l'application : thème, langue, échelle UI.
// Charge depuis SQLite via get_app_prefs au démarrage — avant le premier rendu.
// Persiste chaque changement via save_app_pref.
// Délègue l'application du thème à theme.svelte.ts et la locale à i18n.svelte.ts.

import { getAppPrefs, getAppPref, saveAppPref, type AppPrefs } from '$lib/tauri'
import { initLocale, type Locale } from '$lib/i18n.svelte'
import { applyTheme, type Theme } from './theme.svelte'

/** Base font-size in px used to compute the scaled value (browser default). */
const BASE_FONT_SIZE_PX = 16

// ── State ─────────────────────────────────────────────────────────────────────

let prefs = $state<AppPrefs | null>(null)

// ── Public API ────────────────────────────────────────────────────────────────

/**
 * Load all preferences from SQLite, then apply theme, locale, and UI scale.
 * Must be called once at app startup before the first render.
 */
export async function loadAppPrefs(): Promise<void> {
  prefs = await getAppPrefs()

  initLocale(prefs.locale as Locale)
  applyTheme(prefs.theme as Theme)
  applyScale(prefs.uiScale)
}

/** The loaded preferences, or null before `loadAppPrefs()` resolves. */
export function getPrefs(): AppPrefs | null {
  return prefs
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

// ── Theme ─────────────────────────────────────────────────────────────────────

/**
 * Change the active theme, apply it to the DOM, and persist to SQLite.
 * @param theme `'light'` | `'dark'` | `'system'`
 */
export async function setTheme(theme: Theme): Promise<void> {
  applyTheme(theme)
  await saveAppPref('theme', theme)
  if (prefs) prefs = { ...prefs, theme }
}

// ── UI scale ──────────────────────────────────────────────────────────────────

/**
 * Change the UI scale, apply it to the document, and persist to SQLite.
 *
 * The scale is applied as: `html.style.fontSize = BASE_FONT_SIZE_PX * scale + 'px'`
 * All rem values in components scale proportionally.
 *
 * @param scale Float multiplier — 1.0 = default, 1.2 = 20% larger, 0.9 = 10% smaller
 */
export async function setUiScale(scale: number): Promise<void> {
  applyScale(scale)
  await saveAppPref('ui_scale', String(scale))
  if (prefs) prefs = { ...prefs, uiScale: scale }
}

// ── Favorites ─────────────────────────────────────────────────────────────────

let favorites = $state<string[]>([])

/** Load pinned folder paths from SQLite. Call once at startup. */
export async function loadFavorites(): Promise<void> {
  const raw = await getAppPref('favorites')
  try { favorites = raw ? JSON.parse(raw) : [] } catch { favorites = [] }
}

/** All pinned folder paths. */
export function getFavorites(): string[] { return favorites }

/** Add a path to favorites (no-op if already present). */
export async function addFavorite(path: string): Promise<void> {
  if (favorites.includes(path)) return
  favorites = [...favorites, path]
  await saveAppPref('favorites', JSON.stringify(favorites))
}

/** Remove a path from favorites. */
export async function removeFavorite(path: string): Promise<void> {
  favorites = favorites.filter(p => p !== path)
  await saveAppPref('favorites', JSON.stringify(favorites))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

function applyScale(scale: number): void {
  document.documentElement.style.fontSize = `${BASE_FONT_SIZE_PX * scale}px`
}
