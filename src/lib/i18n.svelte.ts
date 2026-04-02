// Store i18n custom basé sur les runes Svelte 5.
// Expose t(key) pour les traductions en dot notation (ex: t('browser.views.list')).
// Expose setLocale(l) pour changer la langue active et la persister dans SQLite.
// Charge les fichiers JSON depuis src/lib/locales/.

import { saveAppPref } from '$lib/tauri'
import en from './locales/en.json'
import fr from './locales/fr.json'
import ka from './locales/ka.json'

// ── Types ─────────────────────────────────────────────────────────────────────

export type Locale = 'en' | 'fr' | 'ka'

// ── Dictionaries ──────────────────────────────────────────────────────────────

const dictionaries: Record<Locale, Record<string, unknown>> = { en, fr, ka }

// ── State ─────────────────────────────────────────────────────────────────────

let locale = $state<Locale>('en')

// ── Public API ────────────────────────────────────────────────────────────────

/** Return the currently active locale. */
export function getLocale(): Locale {
  return locale
}

/**
 * Set the active locale without persisting to SQLite.
 * Call this during app startup when restoring the saved locale.
 */
export function initLocale(l: Locale): void {
  locale = l
}

/**
 * Change the active locale and persist the choice to SQLite.
 * Call this when the user explicitly picks a language.
 */
export function setLocale(l: Locale): void {
  locale = l
  saveAppPref('locale', l).catch(console.error)
}

/**
 * Translate a key using dot notation.
 *
 * Returns the key itself if no translation is found — never throws.
 *
 * @example
 *   t('browser.views.list')  // → "List" | "Liste" | "სია"
 *   t('status.items')        // → "{n} items" (caller replaces {n})
 */
export function t(key: string): string {
  const parts = key.split('.')
  let node: unknown = dictionaries[locale]

  for (const part of parts) {
    if (node == null || typeof node !== 'object') return key
    node = (node as Record<string, unknown>)[part]
  }

  return typeof node === 'string' ? node : key
}
