<!-- BrowserView.svelte : vue principale du navigateur de fichiers.
     Orchestre Breadcrumb, ViewToggle et la vue active (FileListList / FileListColumn / FileListDetail).
     Délègue toute la logique métier aux stores navigationStore et filesStore. -->

<script lang="ts">
  import Breadcrumb    from '../ui/Breadcrumb.svelte'
  import ViewToggle    from './ViewToggle.svelte'
  import FileListList  from './FileListList.svelte'
  import FileListColumn from './FileListColumn.svelte'
  import FileListDetail from './FileListDetail.svelte'
  import PreviewPanel  from '../preview/PreviewPanel.svelte'

  import { getCurrentPath, canGoBack, canGoForward, back, forward } from '../../stores/navigation.svelte'
  import { loadDir, isLoading, getIndexedAt }                       from '../../stores/files.svelte'
  import { loadFolderPrefs, getPrefs }                              from '../../stores/folderPrefs.svelte'
  import { rescanDir }                                              from '$lib/tauri'
  import { t }                                                      from '$lib/i18n.svelte'
  import type { FileEntry }                                         from '$lib/tauri'

  // ── Load directory when path changes ─────────────────────────────────────
  $effect(() => {
    const path = getCurrentPath()
    if (!path) return
    loadFolderPrefs(path).catch(console.error)
    loadDir(path).catch(console.error)
  })

  // ── Preview panel ─────────────────────────────────────────────────────────
  let previewEntry = $state<FileEntry | null>(null)

  // ── Rescan ────────────────────────────────────────────────────────────────
  let rescanning = $state(false)

  async function handleRescan() {
    const path = getCurrentPath()
    if (!path || rescanning) return
    rescanning = true
    await rescanDir(path).catch(console.error)
    rescanning = false
  }

  // ── Scan time display ─────────────────────────────────────────────────────
  function formatAge(ts: number | null): string {
    if (ts == null) return ''
    const mins = Math.round((Date.now() / 1000 - ts) / 60)
    if (mins < 1) return '< 1 min'
    if (mins < 60) return `${mins} min`
    return `${Math.round(mins / 60)} h`
  }
</script>

<div class="browser-view">
  <!-- Toolbar -->
  <div class="bv-toolbar">
    <button
      class="bv-nav-btn"
      disabled={!canGoBack()}
      onclick={back}
      title="Back"
    >‹</button>

    <button
      class="bv-nav-btn"
      disabled={!canGoForward()}
      onclick={forward}
      title="Forward"
    >›</button>

    <Breadcrumb />

    <div class="bv-scan-status">
      {#if isLoading() || rescanning}
        <span class="bv-scanning">{t('browser.toolbar.scanning')}</span>
      {:else}
        {@const age = formatAge(getIndexedAt())}
        {#if age}
          <span class="bv-scan-time">{age} {t('browser.toolbar.scanned_ago')}</span>
        {/if}
        <button
          class="bv-rescan-btn"
          onclick={handleRescan}
          title={t('browser.toolbar.rescan')}
        >↺</button>
      {/if}
    </div>

    <ViewToggle />
  </div>

  <!-- Content area + preview panel -->
  <div class="bv-content">
    {#if getPrefs().viewMode === 'list'}
      <FileListList   onpreview={(e) => previewEntry = e} />
    {:else if getPrefs().viewMode === 'column'}
      <FileListColumn onpreview={(e) => previewEntry = e} />
    {:else}
      <FileListDetail onpreview={(e) => previewEntry = e} />
    {/if}

    <PreviewPanel
      entry={previewEntry}
      onclose={() => previewEntry = null}
    />
  </div>
</div>

<style>
  .browser-view {
    flex:           1;
    display:        flex;
    flex-direction: column;
    overflow:       hidden;
    min-width:      0;
  }

  /* Toolbar */
  .bv-toolbar {
    display:       flex;
    align-items:   center;
    gap:           var(--space-xs);
    height:        var(--header-row-h, 2.5rem);
    padding:       0 var(--space-sm);
    border-bottom: 1px solid var(--border);
    background:    var(--bg-toolbar);
    flex-shrink:   0;
  }

  .bv-nav-btn {
    background:    none;
    border:        none;
    cursor:        pointer;
    font-size:     1.2rem;
    color:         var(--text-muted);
    padding:       0 var(--space-xs);
    line-height:   1;
    border-radius: var(--radius-sm);
    transition:    color var(--transition-fast);
  }

  .bv-nav-btn:disabled { opacity: 0.3; cursor: default; }
  .bv-nav-btn:not(:disabled):hover { color: var(--text); }

  .bv-scan-status {
    display:     flex;
    align-items: center;
    gap:         var(--space-xs);
    margin-left: auto;
    flex-shrink: 0;
  }

  .bv-scanning,
  .bv-scan-time {
    font-size: var(--font-size-xs);
    color:     var(--text-muted);
  }

  .bv-rescan-btn {
    background:  none;
    border:      none;
    cursor:      pointer;
    font-size:   1rem;
    color:       var(--text-muted);
    padding:     0;
    line-height: 1;
    transition:  color var(--transition-fast);
  }
  .bv-rescan-btn:hover { color: var(--text); }

  /* Content */
  .bv-content {
    flex:     1;
    display:  flex;
    overflow: hidden;
    min-height: 0;
  }
</style>
