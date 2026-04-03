<!-- FileListDetail.svelte : vue détail avec colonnes nom, taille, date de modification, type MIME.
     Utilise VirtualList avec itemHeight=40px.
     Les en-têtes de colonnes sont cliquables pour trier via folderPrefsStore. -->

<script lang="ts">
  import VirtualList from '$lib/VirtualList.svelte'
  import FileItem    from './FileItem.svelte'
  import { getEntries }                   from '../../stores/files.svelte'
  import { getPrefs, setSortBy, setSortDir } from '../../stores/folderPrefs.svelte'
  import { getFileTags }                  from '../../stores/tags.svelte'
  import { navigate }                     from '../../stores/navigation.svelte'
  import { t }                            from '$lib/i18n.svelte'
  import type { FileEntry }               from '$lib/tauri'

  interface Props {
    onpreview?: (entry: FileEntry) => void
  }

  const { onpreview }: Props = $props()

  // ── Container height ──────────────────────────────────────────────────────
  let containerEl = $state<HTMLElement | null>(null)
  let containerH  = $state(0)

  $effect(() => {
    if (!containerEl) return
    const ro = new ResizeObserver(([e]) => { containerH = e.contentRect.height })
    ro.observe(containerEl)
    return () => ro.disconnect()
  })

  // ── Item height from rem ──────────────────────────────────────────────────
  const ROW_REM = 2.5
  let itemHeight = $state(40)

  $effect(() => {
    const update = () => {
      itemHeight = parseFloat(getComputedStyle(document.documentElement).fontSize) * ROW_REM
    }
    update()
    const obs = new MutationObserver(update)
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ['style'] })
    return () => obs.disconnect()
  })

  // ── Sorting ───────────────────────────────────────────────────────────────
  const sortedEntries = $derived(sortEntries(getEntries(), getPrefs().sortBy, getPrefs().sortDir))

  function sortEntries(entries: FileEntry[], sortBy: string, sortDir: string): FileEntry[] {
    return [...entries].sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1
      let cmp = 0
      if (sortBy === 'name') cmp = a.name.localeCompare(b.name)
      else if (sortBy === 'size') cmp = (a.size ?? 0) - (b.size ?? 0)
      else if (sortBy === 'date') cmp = (a.modifiedAt ?? 0) - (b.modifiedAt ?? 0)
      return sortDir === 'asc' ? cmp : -cmp
    })
  }

  /** Handle a column header click — toggle direction or set new sort column. */
  async function handleHeaderClick(col: 'name' | 'size' | 'date') {
    const prefs = getPrefs()
    if (prefs.sortBy === col) {
      await setSortDir(prefs.sortDir === 'asc' ? 'desc' : 'asc')
    } else {
      await setSortBy(col)
      await setSortDir('asc')
    }
  }

  /** Return the sort indicator for a column header. */
  function sortIndicator(col: string): string {
    const prefs = getPrefs()
    if (prefs.sortBy !== col) return ''
    return prefs.sortDir === 'asc' ? ' ↑' : ' ↓'
  }

  // ── Selection & keyboard navigation ──────────────────────────────────────
  let selectedIndex = $state(-1)

  function selectIndex(i: number) {
    selectedIndex = Math.max(0, Math.min(sortedEntries.length - 1, i))
  }

  function handleKeydown(e: KeyboardEvent) {
    if (sortedEntries.length === 0) return
    if (e.key === 'ArrowDown')       { e.preventDefault(); selectIndex(selectedIndex + 1) }
    else if (e.key === 'ArrowUp')    { e.preventDefault(); selectIndex(selectedIndex - 1) }
    else if (e.key === 'Enter' && selectedIndex >= 0) {
      e.preventDefault()
      openEntry(sortedEntries[selectedIndex])
    }
  }

  function openEntry(entry: FileEntry) {
    if (entry.isDir) navigate(entry.path).catch(console.error)
  }

  // ── Tag pre-fetch ─────────────────────────────────────────────────────────
  let tagCache = $state<Record<string, import('$lib/tauri').Tag[]>>({})

  $effect(() => {
    for (const entry of sortedEntries) {
      if (!(entry.path in tagCache)) {
        getFileTags(entry.path)
          .then(tags => { tagCache = { ...tagCache, [entry.path]: tags } })
          .catch(() => {})
      }
    }
  })
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="detail-container"
  bind:this={containerEl}
  role="grid"
  tabindex="0"
  onkeydown={handleKeydown}
>
  <!-- Sticky sortable column headers -->
  <div class="list-header" role="row">
    <span class="lh-icon"></span>

    <button class="lh-name lh-btn" onclick={() => handleHeaderClick('name')}>
      {t('browser.labels.name')}{sortIndicator('name')}
    </button>

    <span class="lh-tags">{t('browser.labels.tags')}</span>

    <button class="lh-size lh-btn" onclick={() => handleHeaderClick('size')}>
      {t('browser.labels.size')}{sortIndicator('size')}
    </button>

    <button class="lh-date lh-btn" onclick={() => handleHeaderClick('date')}>
      {t('browser.labels.date')}{sortIndicator('date')}
    </button>

    <span class="lh-type">{t('browser.labels.type')}</span>
  </div>

  {#if containerH > 0}
    <VirtualList items={sortedEntries} {itemHeight} containerHeight={containerH}>
      {#snippet children({ item: entry, index })}
        <FileItem
          {entry}
          selected={index === selectedIndex}
          tags={tagCache[entry.path] ?? []}
          onclick={() => selectedIndex = index}
          ondblclick={() => openEntry(entry)}
          {onpreview}
        />
      {/snippet}
    </VirtualList>
  {/if}
</div>

<style>
  .detail-container {
    flex:           1;
    overflow:       hidden;
    outline:        none;
    display:        flex;
    flex-direction: column;
  }

  /* Column header row */
  .list-header {
    display:         flex;
    align-items:     center;
    gap:             var(--space-sm);
    height:          var(--header-row-h);
    padding:         0 var(--sidebar-pad-x);
    border-bottom:   1px solid var(--border);
    background:      var(--bg-toolbar);
    flex-shrink:     0;
  }

  /* Clickable header button — inherits layout from its column class */
  .lh-btn {
    background:      none;
    border:          none;
    cursor:          pointer;
    font-family:     inherit;
    font-size:       var(--font-size-xs);
    font-weight:     600;
    text-transform:  uppercase;
    letter-spacing:  0.04em;
    color:           var(--text-muted);
    padding:         0;
    text-align:      left;
    transition:      color var(--transition-fast);
  }
  .lh-btn:hover { color: var(--text-dim); }

  /* Non-sortable header cell */
  .list-header span {
    font-size:       var(--font-size-xs);
    font-weight:     600;
    text-transform:  uppercase;
    letter-spacing:  0.04em;
    color:           var(--text-muted);
  }

  /* Column widths — mirror FileItem */
  .lh-icon { width: 1.25rem; flex-shrink: 0; }
  .lh-name { flex: 1; overflow: hidden; }
  .lh-tags { width: 7.5rem; flex-shrink: 0; }
  .lh-size { width: 4.375rem; text-align: right; flex-shrink: 0; }
  .lh-date { width: 7.5rem; flex-shrink: 0; }
  .lh-type { width: 5.625rem; flex-shrink: 0; }
</style>
