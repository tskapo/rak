<!-- LangPickerModal.svelte : overlay plein-écran affiché au premier lancement.
     Propose les 3 langues dans leur propre script — aucune traduction nécessaire.
     Émet 'pick' avec la locale choisie. Le parent ferme le modal. -->

<script lang="ts">
  import type { Locale } from '$lib/i18n.svelte'

  interface Props {
    onpick: (locale: Locale) => void
  }

  const { onpick }: Props = $props()

  const languages: { locale: Locale; label: string; flag: string }[] = [
    { locale: 'fr', label: 'Français',  flag: '🇫🇷' },
    { locale: 'en', label: 'English',   flag: '🇬🇧' },
    { locale: 'ka', label: 'ქართული',   flag: '🇬🇪' },
  ]
</script>

<div class="overlay" role="dialog" aria-modal="true">
  <div class="card">
    <div class="logo">räk</div>
    <p class="subtitle">Choose your language · Choisissez votre langue · აირჩიეთ ენა</p>

    <div class="lang-list">
      {#each languages as { locale, label, flag }}
        <button class="lang-btn" onclick={() => onpick(locale)}>
          <span class="flag">{flag}</span>
          <span class="label">{label}</span>
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
  }

  .card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
    padding: 2.5rem 3rem;
    background: var(--bg-toolbar);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
    min-width: 20rem;
  }

  .logo {
    font-size: 2rem;
    font-weight: 700;
    letter-spacing: -0.03em;
    color: var(--text);
  }

  .subtitle {
    font-size: 0.8rem;
    color: var(--text-muted);
    text-align: center;
    line-height: 1.5;
  }

  .lang-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
  }

  .lang-btn {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.65rem 1rem;
    background: var(--bg-hover);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text);
    font-size: 0.95rem;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.1s, border-color 0.1s;
    width: 100%;
    text-align: left;
  }

  .lang-btn:hover {
    background: var(--bg-sel);
    border-color: var(--accent);
  }

  .flag {
    font-size: 1.2rem;
  }
</style>
