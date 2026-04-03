<!-- FileListList.svelte : vue liste du contenu d'un dossier.
     Utilise VirtualList avec itemHeight=40px pour scroller des milliers d'entrées sans lag.
     Rend chaque entrée via FileItem.svelte.
     Navigation clavier : flèches ↑↓, Entrée pour ouvrir, Espace pour preview. -->

<script lang="ts">
  import VirtualList from '$lib/VirtualList.svelte'
  import FileItem    from './FileItem.svelte'
  import { getEntries, isLoading, isIndexing } from '../../stores/files.svelte'
  import { getPrefs }   from '../../stores/folderPrefs.svelte'
  import { getFileTags } from '../../stores/tags.svelte'
  import { navigate }   from '../../stores/navigation.svelte'
  import { t }          from '$lib/i18n.svelte'
  import type { FileEntry } from '$lib/tauri'

  interface Props {
    activeTagIds?:  number[]
    previewActive?: boolean
    onpreview?:     (entry: FileEntry) => void
  }

  const { activeTagIds = [], previewActive = false, onpreview }: Props = $props()

  // ── Container height for VirtualList ─────────────────────────────────────
  let containerEl  = $state<HTMLElement | null>(null)
  let containerH   = $state(0)

  $effect(() => {
    if (!containerEl) return
    const ro = new ResizeObserver(([e]) => { containerH = e.contentRect.height })
    ro.observe(containerEl)
    return () => ro.disconnect()
  })

  // ── Item height from rem ──────────────────────────────────────────────────
  // 2.5rem × current html font-size in px
  const ROW_REM = 2.5

  let itemHeight = $state(40)

  $effect(() => {
    const update = () => {
      itemHeight = parseFloat(getComputedStyle(document.documentElement).fontSize) * ROW_REM
    }
    update()
    // Re-compute if ui_scale changes (font-size changes on <html>)
    const obs = new MutationObserver(update)
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ['style'] })
    return () => obs.disconnect()
  })

  // ── Per-row tag loading ───────────────────────────────────────────────────
  let tagCache = $state<Record<string, import('$lib/tauri').Tag[]>>({})

  // ── Sorting + tag filter ─────────────────────────────────────────────────
  const sortedEntries = $derived.by(() => {
    const sorted = sortEntries(getEntries(), getPrefs().sortBy, getPrefs().sortDir)
    if (activeTagIds.length === 0) return sorted
    return sorted.filter(e => {
      const tags = tagCache[e.path] ?? []
      return activeTagIds.every(id => tags.some(t => t.id === id))
    })
  })

  function sortEntries(entries: FileEntry[], sortBy: string, sortDir: string): FileEntry[] {
    return [...entries].sort((a, b) => {
      // Directories always float to the top
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1

      let cmp = 0
      if (sortBy === 'name') cmp = a.name.localeCompare(b.name)
      else if (sortBy === 'size') cmp = (a.size ?? 0) - (b.size ?? 0)
      else if (sortBy === 'date') cmp = (a.modifiedAt ?? 0) - (b.modifiedAt ?? 0)

      return sortDir === 'asc' ? cmp : -cmp
    })
  }

  // ── Selection & keyboard navigation ──────────────────────────────────────
  let selectedIndex = $state<number>(-1)

  function selectIndex(i: number) {
    selectedIndex = Math.max(0, Math.min(sortedEntries.length - 1, i))
  }

  function handleKeydown(e: KeyboardEvent) {
    if (sortedEntries.length === 0) return

    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectIndex(selectedIndex + 1)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectIndex(selectedIndex - 1)
    } else if (e.key === 'Enter' && selectedIndex >= 0) {
      e.preventDefault()
      openEntry(sortedEntries[selectedIndex])
    }
  }

  function openEntry(entry: FileEntry) {
    if (entry.isDir) navigate(entry.path).catch(console.error)
  }

  // Keep preview in sync with keyboard selection when the panel is open.
  $effect(() => {
    if (previewActive && selectedIndex >= 0 && sortedEntries[selectedIndex]) {
      onpreview?.(sortedEntries[selectedIndex])
    }
  })

  // Pre-fetch tags for ALL entries (not just sorted/filtered) so the tag filter
  // has data to work with even before the user applies a filter.
  $effect(() => {
    for (const entry of getEntries()) {
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
  class="list-container"
  bind:this={containerEl}
  role="grid"
  tabindex="0"
  onkeydown={handleKeydown}
>
  {#if (isLoading() || isIndexing()) && sortedEntries.length === 0}
    <div class="list-status">{t('browser.toolbar.indexing')}</div>
  {:else if containerH > 0}
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
  .list-container {
    flex:     1;
    overflow: hidden;
    outline:  none;
    display:  flex;
    flex-direction: column;
  }

  .list-status {
    padding:    var(--space-sm);
    color:      var(--text-muted);
    font-size:  var(--font-size-sm);
    font-style: italic;
  }
</style>
