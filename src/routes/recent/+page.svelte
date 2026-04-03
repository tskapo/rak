<!-- +page.svelte : route /recent — liste des 50 dernières visites.
     Affiche les entrées via getRecents(50). Clic → navigate(). -->

<script lang="ts">
  import AppShell    from '../../components/layout/AppShell.svelte'
  import { getRecents, type FileEntry } from '$lib/tauri'
  import { navigate } from '../../stores/navigation.svelte'
  import { loadTags } from '../../stores/tags.svelte'
  import { loadFavorites } from '../../stores/appPrefs.svelte'
  import { t } from '$lib/i18n.svelte'
  import { onMount } from 'svelte'

  let recents = $state<FileEntry[]>([])

  onMount(async () => {
    await Promise.all([loadTags(), loadFavorites()])
    recents = await getRecents(50).catch(() => [])
  })

  function icon(entry: FileEntry): string {
    if (entry.isDir) return '📁'
    const m = entry.mimeType ?? ''
    if (m.startsWith('image/')) return '🖼'
    if (m.startsWith('video/')) return '🎬'
    if (m.startsWith('audio/')) return '🎵'
    if (m === 'application/pdf') return '📕'
    return '📄'
  }

  function formatDate(ts?: number): string {
    if (ts == null) return '—'
    return new Date(ts * 1000).toLocaleDateString(undefined, {
      day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit',
    })
  }
</script>

<AppShell>
  <div class="recents-page">
    <h1 class="rp-title">{t('sidebar.recents')}</h1>
    <ul class="rp-list">
      {#each recents as entry (entry.path)}
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
        <li
          class="rp-item"
          role="option"
          aria-selected="false"
          onclick={() => { if (entry.isDir) navigate(entry.path).catch(console.error) }}
        >
          <span class="rp-icon">{icon(entry)}</span>
          <span class="rp-name">{entry.name}</span>
          <span class="rp-path">{entry.path}</span>
          <span class="rp-date">{formatDate(entry.modifiedAt)}</span>
        </li>
      {:else}
        <li class="rp-empty">—</li>
      {/each}
    </ul>
  </div>
</AppShell>

<style>
  .recents-page {
    flex:           1;
    display:        flex;
    flex-direction: column;
    padding:        var(--space-sm);
    overflow-y:     auto;
    min-width:      0;
  }

  .rp-title {
    font-size:   var(--font-size-base);
    font-weight: 600;
    color:       var(--text);
    margin:      0 0 var(--space-sm);
  }

  .rp-list {
    list-style: none;
    margin:     0;
    padding:    0;
  }

  .rp-item {
    display:       grid;
    grid-template-columns: 1.5rem 1fr 2fr auto;
    align-items:   center;
    gap:           var(--space-sm);
    padding:       var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
    cursor:        pointer;
    transition:    background var(--transition-fast);
  }

  .rp-item:hover { background: var(--bg-hover); }

  .rp-icon { text-align: center; }

  .rp-name {
    font-size:     var(--font-size-base);
    color:         var(--text);
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
  }

  .rp-path {
    font-size:     var(--font-size-xs);
    color:         var(--text-muted);
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
  }

  .rp-date {
    font-size:  var(--font-size-xs);
    color:      var(--text-muted);
    white-space: nowrap;
  }

  .rp-empty {
    padding:   var(--space-sm);
    color:     var(--text-muted);
    font-size: var(--font-size-sm);
  }
</style>
