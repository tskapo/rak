<!-- FileItem.svelte : rendu d'un seul fichier ou dossier dans n'importe quelle vue liste.
     Props : entry (FileEntry), selected (boolean).
     Callbacks : onclick (sélection), ondblclick (ouverture/navigation), onpreview (espace). -->

<script lang="ts">
  import type { FileEntry, Tag } from '$lib/tauri'
  import TagBadge  from '../ui/TagBadge.svelte'
  import TagPicker from '../ui/TagPicker.svelte'
  import { t }     from '$lib/i18n.svelte'

  interface Props {
    entry:      FileEntry
    selected?:  boolean
    tags?:      Tag[]
    onclick?:   (entry: FileEntry) => void
    ondblclick?: (entry: FileEntry) => void
    onpreview?: (entry: FileEntry) => void
  }

  const {
    entry,
    selected  = false,
    tags      = [],
    onclick,
    ondblclick,
    onpreview,
  }: Props = $props()

  /** Icon emoji based on MIME type or directory flag. */
  const icon = $derived(getIcon(entry))

  /** CSS custom-property name for the icon colour. */
  const iconColor = $derived(getIconColor(entry))

  function getIcon(e: FileEntry): string {
    if (e.isDir) return '📁'
    const m = e.mimeType ?? ''
    if (m.startsWith('image/'))  return '🖼'
    if (m.startsWith('video/'))  return '🎬'
    if (m.startsWith('audio/'))  return '🎵'
    if (m === 'application/pdf') return '📕'
    if (m.startsWith('text/'))   return '📄'
    return '📄'
  }

  function getIconColor(e: FileEntry): string {
    if (e.isDir) return 'var(--icon-dir)'
    const m = e.mimeType ?? ''
    if (m.startsWith('image/')) return 'var(--icon-img)'
    if (m.startsWith('video/')) return 'var(--icon-video)'
    return 'var(--icon-doc)'
  }

  /** Format bytes → human-readable size string. */
  function formatSize(bytes: number | undefined): string {
    if (bytes == null) return '—'
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(0)} Ko`
    if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} Mo`
    return `${(bytes / 1024 ** 3).toFixed(2)} Go`
  }

  /** Format Unix timestamp → locale date string. */
  function formatDate(ts: number | undefined): string {
    if (ts == null) return '—'
    return new Date(ts * 1000).toLocaleDateString(undefined, {
      day: 'numeric', month: 'short', year: 'numeric',
    })
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === ' ') {
      e.preventDefault()
      onpreview?.(entry)
    }
  }

  // ── Context menu ──────────────────────────────────────────────────────────
  let menuX        = $state(0)
  let menuY        = $state(0)
  let showMenu     = $state(false)
  let showTagPicker = $state(false)

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault()
    menuX    = e.clientX
    menuY    = e.clientY
    showMenu = true
  }

  function closeMenu() { showMenu = false }

  function openTagPicker() {
    showMenu      = false
    showTagPicker = true
  }
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
  class="file-item"
  class:selected
  role="row"
  tabindex="0"
  onclick={() => onclick?.(entry)}
  ondblclick={() => ondblclick?.(entry)}
  onkeydown={handleKeydown}
  oncontextmenu={handleContextMenu}
>
  <span class="file-icon" style:color={iconColor}>{icon}</span>

  <span class="file-name">{entry.name}</span>

  {#if tags.length > 0}
    <span class="file-tags">
      {#each tags as tag (tag.id)}
        <TagBadge {tag} />
      {/each}
    </span>
  {/if}

  <span class="file-size">{formatSize(entry.size)}</span>
  <span class="file-date">{formatDate(entry.modifiedAt)}</span>
  <span class="file-type">{entry.mimeType?.split('/')[1] ?? (entry.isDir ? 'Folder' : '—')}</span>
</div>

{#if showMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="ctx-backdrop" role="presentation" onclick={closeMenu}></div>
  <menu class="ctx-menu" style:left="{menuX}px" style:top="{menuY}px">
    <li><button class="ctx-item" onclick={openTagPicker}>{t('actions.tag')}</button></li>
  </menu>
{/if}

{#if showTagPicker}
  <TagPicker {entry} x={menuX} y={menuY} onclose={() => { showTagPicker = false }} />
{/if}

<style>
  .file-item {
    display:     flex;
    align-items: center;
    gap:         var(--space-sm);
    height:      var(--row-h);
    padding:     0 var(--sidebar-pad-x);
    cursor:      pointer;
    outline:     none;
    transition:  background var(--transition-fast);
  }

  .file-item:hover    { background: var(--bg-hover); }
  .file-item.selected { background: var(--bg-sel); }
  .file-item:focus-visible { outline: 1px solid var(--accent); outline-offset: -1px; }

  .file-icon {
    font-size:   1rem;
    width:       1.25rem;
    text-align:  center;
    flex-shrink: 0;
  }

  .file-name {
    flex:          1;
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
    font-size:     var(--font-size-base);
    color:         var(--text);
  }

  .file-tags {
    display:     flex;
    gap:         var(--space-xs);
    flex-shrink: 0;
    align-items: center;
  }

  .file-size,
  .file-date,
  .file-type {
    font-size:   var(--font-size-sm);
    color:       var(--text-dim);
    flex-shrink: 0;
    white-space: nowrap;
  }

  .file-size { width: 4.375rem; text-align: right; }
  .file-date { width: 7.5rem; }
  .file-type { width: 5.625rem; color: var(--text-muted); }

  /* Context menu */
  .ctx-backdrop {
    position: fixed;
    inset:    0;
    z-index:  100;
  }

  .ctx-menu {
    position:      fixed;
    z-index:       101;
    margin:        0;
    padding:       var(--space-xs) 0;
    list-style:    none;
    background:    var(--bg-toolbar);
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    box-shadow:    0 4px 12px rgba(0,0,0,.18);
    min-width:     8rem;
  }

  .ctx-item {
    display:     block;
    width:       100%;
    padding:     var(--space-xs) var(--space-sm);
    background:  none;
    border:      none;
    text-align:  left;
    font-family: inherit;
    font-size:   var(--font-size-base);
    color:       var(--text);
    cursor:      pointer;
  }

  .ctx-item:hover { background: var(--bg-hover); }
</style>
