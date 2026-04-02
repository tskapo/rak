// Applique le thème actif sur <html> via data-theme et écoute matchMedia pour le mode 'system'.
// Piloté par appPrefsStore — ne stocke rien lui-même, ne lit pas localStorage.
// Responsabilité unique : synchroniser le DOM avec la valeur de thème courante.

export type Theme = 'light' | 'dark' | 'system'

// ── Internal state ────────────────────────────────────────────────────────────

// Keep a reference to the active matchMedia + listener so we can clean up
// when the theme changes (e.g. user switches from 'system' to 'dark').
let mediaQuery:   MediaQueryList | null = null
let mediaHandler: (() => void)   | null = null

// ── Public API ────────────────────────────────────────────────────────────────

/**
 * Apply `theme` to the document.
 *
 * - `'light'` / `'dark'` → sets `data-theme` on `<html>` immediately.
 * - `'system'` → reads `prefers-color-scheme` from the OS and follows it
 *   reactively (switches automatically when the user changes OS appearance).
 */
export function applyTheme(theme: Theme): void {
  // Remove any previous matchMedia listener
  if (mediaHandler && mediaQuery) {
    mediaQuery.removeEventListener('change', mediaHandler)
    mediaHandler = null
    mediaQuery   = null
  }

  if (theme === 'system') {
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

    const sync = () => {
      document.documentElement.dataset.theme = mediaQuery!.matches ? 'dark' : 'light'
    }

    sync() // apply immediately
    mediaHandler = sync
    mediaQuery.addEventListener('change', sync)
  } else {
    document.documentElement.dataset.theme = theme
  }
}
