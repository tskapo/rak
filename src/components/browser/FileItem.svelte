<!-- FileItem.svelte : rendu d'un seul fichier ou dossier dans n'importe quelle vue liste.
     Props : entry (FileEntry), selected (boolean).
     Callbacks : onclick (sélection), ondblclick (ouverture/navigation), onpreview (espace). -->

<script lang="ts">
  import type { FileEntry, Tag } from '$lib/tauri'
  import { openPath, deleteEntry } from '$lib/tauri'
  import TagBadge  from '../ui/TagBadge.svelte'
  import TagPicker from '../ui/TagPicker.svelte'
  import { t }     from '$lib/i18n.svelte'
  import { navigate } from '../../stores/navigation.svelte'

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
  let menuX          = $state(0)
  let menuY          = $state(0)
  let showMenu       = $state(false)
  let showTagPicker  = $state(false)
  let confirmingDelete = $state(false)

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault()
    menuX           = e.clientX
    menuY           = e.clientY
    showMenu        = true
    confirmingDelete = false
  }

  function closeMenu() { showMenu = false; confirmingDelete = false }

  function openTagPicker() {
    showMenu      = false
    showTagPicker = true
  }

  async function handleOpen() {
    closeMenu()
    if (entry.isDir) await navigate(entry.path).catch(console.error)
    else             await openPath(entry.path).catch(console.error)
  }

  function handlePreview() {
    closeMenu()
    onpreview?.(entry)
  }

  async function handleCopyPath() {
    closeMenu()
    await navigator.clipboard.writeText(entry.path).catch(console.error)
  }

  async function handleDelete() {
    closeMenu()
    await deleteEntry(entry.path).catch(console.error)
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
    <li><button class="ctx-item" onclick={handleOpen}>{t('actions.open')}</button></li>
    {#if !entry.isDir}
      <li><button class="ctx-item" onclick={handlePreview}>{t('actions.preview')}</button></li>
    {/if}
    <li role="separator" class="ctx-sep"></li>
    <li><button class="ctx-item" onclick={openTagPicker}>{t('actions.tag')}</button></li>
    <li><button class="ctx-item" onclick={handleCopyPath}>{t('actions.copy_path')}</button></li>
    <li role="separator" class="ctx-sep"></li>
    {#if confirmingDelete}
      <li class="ctx-confirm">
        <span class="ctx-confirm-label">{t('actions.delete')} ?</span>
        <button class="ctx-btn ctx-btn-danger" onclick={handleDelete}>{t('actions.ok')}</button>
        <button class="ctx-btn" onclick={() => confirmingDelete = false}>{t('actions.cancel')}</button>
      </li>
    {:else}
      <li><button class="ctx-item ctx-item-danger" onclick={() => confirmingDelete = true}>{t('actions.delete')}</button></li>
    {/if}
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
  .ctx-item-danger { color: var(--danger, #d9534f); }
  .ctx-item-danger:hover { background: var(--danger-bg, rgba(217,83,79,.08)); }

  .ctx-sep {
    height:     1px;
    margin:     var(--space-xs) 0;
    background: var(--border);
  }

  .ctx-confirm {
    display:     flex;
    align-items: center;
    gap:         var(--space-xs);
    padding:     var(--space-xs) var(--space-sm);
  }

  .ctx-confirm-label {
    flex:      1;
    font-size: var(--font-size-base);
    color:     var(--danger, #d9534f);
  }

  .ctx-btn {
    padding:       0.15rem 0.4rem;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    background:    var(--bg-toolbar);
    font-family:   inherit;
    font-size:     var(--font-size-sm);
    color:         var(--text);
    cursor:        pointer;
  }

  .ctx-btn-danger {
    background: var(--danger, #d9534f);
    border-color: var(--danger, #d9534f);
    color: #fff;
  }
</style>
