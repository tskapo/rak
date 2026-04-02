// Store des préférences globales de l'application : thème et langue.
// Charge depuis SQLite via invoke('get_app_prefs') au démarrage — avant le premier rendu.
// Persiste chaque changement via invoke('save_app_pref', { key, value }).
// La langue peut différer de la locale système ; le thème peut différer des préférences OS.
// Notifie i18n.svelte.ts et themeStore à chaque changement via $effect.
