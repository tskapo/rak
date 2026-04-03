<!-- BrowserView.svelte : vue principale du navigateur de fichiers.
     Orchestre Breadcrumb, ViewToggle et la vue active (FileListList / FileListColumn / FileListDetail).
     Délègue toute la logique métier aux stores navigationStore et filesStore. -->

<script lang="ts">
  import Breadcrumb        from '../ui/Breadcrumb.svelte'
  import ViewToggle        from './ViewToggle.svelte'
  import FileListList      from './FileListList.svelte'
  import FileListColumn    from './FileListColumn.svelte'
  import FileListDetail    from './FileListDetail.svelte'
  import PreviewPanel      from '../preview/PreviewPanel.svelte'
  import TagFilterPopover  from '../ui/TagFilterPopover.svelte'

  import { getCurrentPath, canGoBack, canGoForward, back, forward } from '../../stores/navigation.svelte'
  import { loadDir, isLoading, getIndexedAt, getEntries }           from '../../stores/files.svelte'
  import { loadFolderPrefs, getPrefs }                              from '../../stores/folderPrefs.svelte'
  import { rescanDir }                                              from '$lib/tauri'
  import { t }                                                      from '$lib/i18n.svelte'

  // ── Load directory when path changes ─────────────────────────────────────
  $effect(() => {
    const path = getCurrentPath()
    if (!path) return
    // Await folder prefs first so sort settings are ready before the list renders.
    loadFolderPrefs(path)
      .catch(console.error)
      .then(() => loadDir(path).catch(console.error))
  })

  // ── Preview panel ─────────────────────────────────────────────────────────
  // Store only the path so the entry is always derived from the live store.
  // This ensures the preview reflects updated MIME types after pass-2 enrichment.
  let previewPath  = $state<string | null>(null)
  const previewEntry = $derived(
    previewPath ? (getEntries().find(e => e.path === previewPath) ?? null) : null
  )

  // ── Tag filter ────────────────────────────────────────────────────────────
  let activeTagIds    = $state<number[]>([])
  let showTagFilter   = $state(false)
  let tagBtnEl        = $state<HTMLElement | null>(null)
  let tagPopoverX     = $state(0)
  let tagPopoverY     = $state(0)

  function toggleTagFilter() {
    if (showTagFilter) { showTagFilter = false; return }
    if (tagBtnEl) {
      const r = tagBtnEl.getBoundingClientRect()
      tagPopoverX = r.left
      tagPopoverY = r.bottom + 4
    }
    showTagFilter = true
  }

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
      {/if}
      <button
        class="bv-rescan-btn"
        class:spinning={isLoading() || rescanning}
        disabled={rescanning}
        onclick={handleRescan}
        title={t('browser.toolbar.rescan')}
      >↺</button>
    </div>

    <button
      bind:this={tagBtnEl}
      class="bv-tags-btn"
      class:active={activeTagIds.length > 0}
      onclick={toggleTagFilter}
      title={t('browser.toolbar.tags')}
    >
      {t('browser.toolbar.tags')}
      {#if activeTagIds.length > 0}
        <span class="bv-tags-badge">{activeTagIds.length}</span>
      {/if}
    </button>

    <ViewToggle />
  </div>

  {#if showTagFilter}
    <TagFilterPopover
      {activeTagIds}
      x={tagPopoverX}
      y={tagPopoverY}
      onchange={(ids) => activeTagIds = ids}
      onclose={() => showTagFilter = false}
    />
  {/if}

  <!-- Content area + preview panel -->
  <div class="bv-content">
    {#if getPrefs().viewMode === 'list'}
      <FileListList   {activeTagIds} previewActive={previewPath !== null} onpreview={(e) => previewPath = e.path} />
    {:else if getPrefs().viewMode === 'column'}
      <FileListColumn onpreview={(e) => previewPath = e.path} />
    {:else}
      <FileListDetail {activeTagIds} previewActive={previewPath !== null} onpreview={(e) => previewPath = e.path} />
    {/if}

    <PreviewPanel
      entry={previewEntry}
      onclose={() => previewPath = null}
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
  .bv-rescan-btn.spinning {
    animation: spin 0.8s linear infinite;
    color:     var(--accent);
  }

  .bv-tags-btn {
    display:       flex;
    align-items:   center;
    gap:           0.25rem;
    background:    none;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor:        pointer;
    font-family:   inherit;
    font-size:     var(--font-size-sm);
    color:         var(--text-dim);
    padding:       0.1rem 0.4rem;
    transition:    color var(--transition-fast), border-color var(--transition-fast);
    flex-shrink:   0;
  }

  .bv-tags-btn:hover        { color: var(--text); border-color: var(--text-dim); }
  .bv-tags-btn.active       { color: var(--accent); border-color: var(--accent); }

  .bv-tags-badge {
    display:       inline-flex;
    align-items:   center;
    justify-content: center;
    width:         1rem;
    height:        1rem;
    border-radius: 50%;
    background:    var(--accent);
    color:         #fff;
    font-size:     0.625rem;
    font-weight:   600;
  }

  /* Content */
  .bv-content {
    flex:     1;
    display:  flex;
    overflow: hidden;
    min-height: 0;
  }
</style>
