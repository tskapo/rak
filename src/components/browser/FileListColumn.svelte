<!-- FileListColumn.svelte : vue colonne style Finder (Miller columns).
     Chaque colonne affiche le contenu d'un niveau du dossier.
     Cliquer un dossier ouvre le niveau suivant à droite.
     Utilise VirtualList (itemHeight rem-aware) pour chaque colonne.
     Navigation clavier : ↑↓ dans la colonne, ←→ entre colonnes. -->

<script lang="ts">
  import VirtualList from '$lib/VirtualList.svelte'
  import TagBadge    from '../ui/TagBadge.svelte'
  import { listDir } from '$lib/tauri'
  import { getCurrentPath, navigate } from '../../stores/navigation.svelte'
  import { getFileTags }              from '../../stores/tags.svelte'
  import type { FileEntry, Tag }      from '$lib/tauri'

  interface Props {
    onpreview?: (entry: FileEntry) => void
  }

  const { onpreview }: Props = $props()

  // ── Container height (shared by all columns) ──────────────────────────────
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

  // ── Column paths ──────────────────────────────────────────────────────────
  let columnPaths     = $state<string[]>(getCurrentPath() ? [getCurrentPath()] : [])
  let selectedInCol   = $state<Record<number, number>>({})   // colIndex → item index
  let focusedCol      = $state(0)

  // Reset columns when root changes via back/forward navigation
  $effect(() => {
    const root = getCurrentPath()
    if (root && columnPaths[0] !== root) {
      columnPaths   = [root]
      selectedInCol = {}
      focusedCol    = 0
    }
  })

  // ── Per-column data ───────────────────────────────────────────────────────
  interface ColState { entries: FileEntry[]; loading: boolean; error: string | null }
  let colData = $state<Record<string, ColState>>({})

  $effect(() => {
    for (const path of columnPaths) {
      if (!(path in colData)) {
        colData = { ...colData, [path]: { entries: [], loading: true, error: null } }
        listDir(path)
          .then(r  => { colData = { ...colData, [path]: { entries: r.entries, loading: false, error: null } } })
          .catch(e => { colData = { ...colData, [path]: { entries: [],        loading: false, error: String(e) } } })
      }
    }
  })

  // ── Sorting ───────────────────────────────────────────────────────────────
  function getSorted(path: string): FileEntry[] {
    return [...(colData[path]?.entries ?? [])].sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1
      return a.name.localeCompare(b.name)
    })
  }

  // ── Item interaction ──────────────────────────────────────────────────────
  function handleItemClick(colIndex: number, entryIndex: number, entry: FileEntry) {
    focusedCol    = colIndex
    selectedInCol = { ...selectedInCol, [colIndex]: entryIndex }

    if (entry.isDir) {
      columnPaths = [...columnPaths.slice(0, colIndex + 1), entry.path]
      navigate(entry.path).catch(console.error)
    } else {
      // Trim any deeper columns opened from a previously selected dir
      columnPaths = columnPaths.slice(0, colIndex + 1)
    }
  }

  // ── Keyboard navigation ───────────────────────────────────────────────────
  function handleKeydown(e: KeyboardEvent) {
    const path = columnPaths[focusedCol]
    if (!path) return
    const sorted = getSorted(path)
    if (sorted.length === 0) return

    const cur = selectedInCol[focusedCol] ?? -1

    if (e.key === 'ArrowDown') {
      e.preventDefault()
      const next = Math.min(sorted.length - 1, cur + 1)
      handleItemClick(focusedCol, next, sorted[next])
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      const prev = Math.max(0, cur - 1)
      handleItemClick(focusedCol, prev, sorted[prev])
    } else if (e.key === 'ArrowRight') {
      e.preventDefault()
      if (focusedCol < columnPaths.length - 1) focusedCol += 1
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault()
      if (focusedCol > 0) focusedCol -= 1
    } else if ((e.key === 'Enter' || e.key === ' ') && cur >= 0 && !sorted[cur].isDir) {
      e.preventDefault()
      onpreview?.(sorted[cur])
    }
  }

  // ── Tag cache ─────────────────────────────────────────────────────────────
  let tagCache = $state<Record<string, Tag[]>>({})

  $effect(() => {
    for (const path of columnPaths) {
      for (const entry of (colData[path]?.entries ?? [])) {
        if (!(entry.path in tagCache)) {
          getFileTags(entry.path)
            .then(tags => { tagCache = { ...tagCache, [entry.path]: tags } })
            .catch(() => {})
        }
      }
    }
  })

  // ── Icon helper ───────────────────────────────────────────────────────────
  function getIcon(e: FileEntry): string {
    if (e.isDir) return '📁'
    const m = e.mimeType ?? ''
    if (m.startsWith('image/')) return '🖼'
    if (m.startsWith('video/')) return '🎬'
    if (m.startsWith('audio/')) return '🎵'
    if (m === 'application/pdf') return '📕'
    return '📄'
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="column-browser"
  bind:this={containerEl}
  role="grid"
  tabindex="0"
  onkeydown={handleKeydown}
>
  {#each columnPaths as colPath, colIndex (colPath)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="column"
      class:focused={focusedCol === colIndex}
      role="gridcell"
      tabindex="-1"
      onclick={() => { focusedCol = colIndex }}
    >
      {#if colData[colPath]?.loading}
        <div class="col-status">…</div>
      {:else if colData[colPath]?.error}
        <div class="col-status col-error">{colData[colPath].error}</div>
      {:else if containerH > 0}
        <VirtualList items={getSorted(colPath)} {itemHeight} containerHeight={containerH}>
          {#snippet children({ item: entry, index })}
            <!-- svelte-ignore a11y_interactive_supports_focus -->
            <div
              class="col-item"
              class:selected={selectedInCol[colIndex] === index}
              role="row"
              tabindex="-1"
              onclick={(e) => { e.stopPropagation(); handleItemClick(colIndex, index, entry) }}
              ondblclick={() => { if (!entry.isDir) onpreview?.(entry) }}
            >
              <span class="ci-icon">{getIcon(entry)}</span>
              <span class="ci-name">{entry.name}</span>
              {#if (tagCache[entry.path] ?? []).length > 0}
                <span class="ci-tags">
                  {#each tagCache[entry.path] as tag (tag.id)}
                    <TagBadge {tag} />
                  {/each}
                </span>
              {/if}
              {#if entry.isDir}
                <span class="ci-chevron">›</span>
              {/if}
            </div>
          {/snippet}
        </VirtualList>
      {/if}
    </div>
  {/each}
</div>

<style>
  .column-browser {
    flex:            1;
    display:         flex;
    flex-direction:  row;
    overflow-x:      auto;
    overflow-y:      hidden;
    outline:         none;
  }

  .column {
    min-width:       14rem;
    width:           14rem;
    flex-shrink:     0;
    height:          100%;
    border-right:    1px solid var(--border);
    display:         flex;
    flex-direction:  column;
    overflow:        hidden;
  }

  /* Subtle left accent when this column has keyboard focus */
  .column.focused {
    box-shadow: inset 2px 0 0 var(--accent);
  }

  .col-status {
    padding:    var(--space-sm);
    color:      var(--text-muted);
    font-size:  var(--font-size-sm);
  }

  .col-error { color: var(--danger, #e53e3e); }

  .col-item {
    display:     flex;
    align-items: center;
    gap:         var(--space-sm);
    height:      var(--row-h);
    padding:     0 var(--space-sm);
    cursor:      pointer;
    outline:     none;
    transition:  background var(--transition-fast);
  }

  .col-item:hover    { background: var(--bg-hover); }
  .col-item.selected { background: var(--bg-sel); }
  .col-item:focus-visible { outline: 1px solid var(--accent); outline-offset: -1px; }

  .ci-icon {
    font-size:   1rem;
    width:       1.25rem;
    text-align:  center;
    flex-shrink: 0;
  }

  .ci-name {
    flex:          1;
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
    font-size:     var(--font-size-base);
    color:         var(--text);
  }

  .ci-tags {
    display:     flex;
    gap:         var(--space-xs);
    flex-shrink: 0;
    align-items: center;
  }

  .ci-chevron {
    color:       var(--text-muted);
    font-size:   var(--font-size-sm);
    flex-shrink: 0;
  }
</style>
