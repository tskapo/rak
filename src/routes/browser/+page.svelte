<!-- +page.svelte : route /browser — vue principale du navigateur de fichiers.
     Rend AppShell + BrowserView.
     Lit le paramètre de query ?path= pour permettre les liens directs vers un dossier. -->

<script lang="ts">
  import { page }     from '$app/stores'
  import { onMount }  from 'svelte'
  import AppShell     from '../../components/layout/AppShell.svelte'
  import BrowserView  from '../../components/browser/BrowserView.svelte'
  import { navigate } from '../../stores/navigation.svelte'
  import { loadTags } from '../../stores/tags.svelte'
  import { loadFavorites } from '../../stores/appPrefs.svelte'

  onMount(async () => {
    // Load app-wide data
    await Promise.all([loadTags(), loadFavorites()])

    // Honor ?path= query param (deep link), otherwise navigation store
    // already has the start path from the indexer
    const qpath = $page.url.searchParams.get('path')
    if (qpath) await navigate(qpath).catch(console.error)
  })
</script>

<AppShell>
  <BrowserView />
</AppShell>
