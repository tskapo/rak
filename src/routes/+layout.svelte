<!-- +layout.svelte : layout racine de l'application SvelteKit.
     Charge les préférences SQLite au démarrage, initialise i18n.
     Affiche LangPickerModal au premier lancement avant toute autre UI. -->

<script lang="ts">
  import { onMount } from 'svelte'
  import LangPickerModal from '../components/ui/LangPickerModal.svelte'
  import { loadAppPrefs, isFirstLaunch, completeFirstLaunch } from '../stores/appPrefs.svelte'
  import { setLocale, type Locale } from '$lib/i18n.svelte'

  interface Props { children: import('svelte').Snippet }
  const { children }: Props = $props()

  let ready       = $state(false)
  let firstLaunch = $state(true)

  onMount(async () => {
    await loadAppPrefs()
    firstLaunch = isFirstLaunch()
    ready = true
  })

  async function onLanguagePicked(locale: Locale) {
    setLocale(locale)
    await completeFirstLaunch()
    firstLaunch = false
  }
</script>

{#if !ready}
  <!-- Nothing — avoids flash before prefs are loaded -->
{:else if firstLaunch}
  <LangPickerModal onpick={onLanguagePicked} />
{:else}
  {@render children()}
{/if}
