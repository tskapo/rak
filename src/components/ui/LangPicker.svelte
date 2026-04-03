<!-- LangPicker.svelte : sélecteur de langue de l'interface sous forme de dropdown.
     Affiché dans le footer de la sidebar ou les préférences.
     Appelle setLocale() → persiste en SQLite → notifie i18n.svelte.ts.
     Langues disponibles : Français (fr) | English (en) | ქართული (ka). -->

<script lang="ts">
  import { getLocale, setLocale, type Locale } from '$lib/i18n.svelte'

  const LANGS: { code: Locale; label: string }[] = [
    { code: 'fr', label: 'Français' },
    { code: 'en', label: 'English'  },
    { code: 'ka', label: 'ქართული' },
  ]
</script>

<select
  class="lang-picker"
  value={getLocale()}
  onchange={(e) => setLocale((e.currentTarget as HTMLSelectElement).value as Locale)}
>
  {#each LANGS as lang (lang.code)}
    <option value={lang.code}>{lang.label}</option>
  {/each}
</select>

<style>
  .lang-picker {
    background:    var(--bg-toolbar);
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    color:         var(--text-dim);
    font-family:   inherit;
    font-size:     var(--font-size-xs);
    padding:       0.15rem var(--space-xs);
    cursor:        pointer;
    outline:       none;
    width:         100%;
  }

  .lang-picker:focus { border-color: var(--accent); }
</style>
