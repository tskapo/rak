<!-- PreviewPanel.svelte : panneau d'aperçu déclenché par la barre espace sur un item sélectionné.
     Images : fit-to-screen avec ratio préservé (asset protocol).
     Vidéo / audio : player HTML5 natif.
     PDF : embed natif WebKit.
     Autres : icône MIME + bloc de métadonnées.
     Se ferme via Escape. Slide-in CSS depuis la droite. -->

<script lang="ts">
  import { convertFileSrc }  from '@tauri-apps/api/core'
  import { t }               from '$lib/i18n.svelte'
  import { getFileTags }     from '../../stores/tags.svelte'
  import TagBadge            from '../ui/TagBadge.svelte'
  import type { FileEntry, Tag } from '$lib/tauri'

  interface Props {
    entry:    FileEntry | null
    onclose?: () => void
  }

  const { entry, onclose }: Props = $props()

  // ── Asset URL ─────────────────────────────────────────────────────────────
  const src = $derived(entry ? convertFileSrc(entry.path) : '')

  // ── Type detection ────────────────────────────────────────────────────────
  const mime    = $derived(entry?.mimeType ?? '')
  const isImage = $derived(mime.startsWith('image/'))
  const isVideo = $derived(mime.startsWith('video/'))
  const isAudio = $derived(mime.startsWith('audio/'))
  const isPdf   = $derived(mime === 'application/pdf')

  // ── Tags ──────────────────────────────────────────────────────────────────
  let tags = $state<Tag[]>([])

  $effect(() => {
    if (!entry) { tags = []; return }
    getFileTags(entry.path).then(t => { tags = t }).catch(() => { tags = [] })
  })

  // ── Close on Escape ───────────────────────────────────────────────────────
  $effect(() => {
    if (!entry) return
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') { e.preventDefault(); onclose?.() }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  })

  // ── Metadata helpers ──────────────────────────────────────────────────────
  function formatSize(bytes?: number): string {
    if (bytes == null) return '—'
    if (bytes < 1024)       return `${bytes} B`
    if (bytes < 1024 ** 2)  return `${(bytes / 1024).toFixed(0)} Ko`
    if (bytes < 1024 ** 3)  return `${(bytes / 1024 ** 2).toFixed(1)} Mo`
    return `${(bytes / 1024 ** 3).toFixed(2)} Go`
  }

  function formatDate(ts?: number): string {
    if (ts == null) return '—'
    return new Date(ts * 1000).toLocaleDateString(undefined, {
      day: 'numeric', month: 'short', year: 'numeric',
    })
  }

  function getBigIcon(e: FileEntry): string {
    if (e.isDir) return '📁'
    const m = e.mimeType ?? ''
    if (m.startsWith('image/'))  return '🖼'
    if (m.startsWith('video/'))  return '🎬'
    if (m.startsWith('audio/'))  return '🎵'
    if (m === 'application/pdf') return '📕'
    if (m.startsWith('text/'))   return '📄'
    return '📄'
  }
</script>

<div class="preview-panel" class:open={!!entry}>
  {#if entry}
    <!-- Header -->
    <div class="pp-header">
      <span class="pp-name" title={entry.name}>{entry.name}</span>
      <button class="pp-close" onclick={() => onclose?.()}>✕</button>
    </div>

    <!-- Media / content area -->
    <div class="pp-body">
      {#if isImage}
        <div class="pp-img-wrap">
          <img class="pp-image" src={src} alt={entry.name} />
        </div>

      {:else if isVideo}
        <!-- svelte-ignore a11y_media_has_caption -->
        <video class="pp-video" src={src} controls></video>

      {:else if isAudio}
        <div class="pp-audio-wrap">
          <span class="pp-big-icon">🎵</span>
          <!-- svelte-ignore a11y_media_has_caption -->
          <audio src={src} controls class="pp-audio"></audio>
        </div>

      {:else if isPdf}
        <embed class="pp-pdf" src={src} type="application/pdf" />

      {:else}
        <!-- Generic: big icon + metadata -->
        <div class="pp-generic">
          <span class="pp-big-icon">{getBigIcon(entry)}</span>
        </div>
      {/if}
    </div>

    <!-- Metadata footer -->
    <div class="pp-meta">
      <dl class="pp-meta-list">
        <dt>{t('browser.labels.name')}</dt>
        <dd class="pp-meta-name">{entry.name}</dd>

        <dt>{t('browser.labels.size')}</dt>
        <dd>{formatSize(entry.size)}</dd>

        <dt>{t('browser.labels.date')}</dt>
        <dd>{formatDate(entry.modifiedAt)}</dd>

        <dt>{t('browser.labels.type')}</dt>
        <dd>{entry.mimeType ?? (entry.isDir ? 'Folder' : '—')}</dd>
      </dl>

      {#if tags.length > 0}
        <div class="pp-tags">
          {#each tags as tag (tag.id)}
            <TagBadge {tag} />
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .preview-panel {
    width:          0;
    flex-shrink:    0;
    overflow:       hidden;
    border-left:    1px solid var(--border);
    background:     var(--bg-sidebar, var(--bg));
    display:        flex;
    flex-direction: column;
    transition:     width var(--transition-fast, 150ms ease);
  }

  .preview-panel.open {
    width: 18rem;
  }

  /* Header */
  .pp-header {
    display:       flex;
    align-items:   center;
    gap:           var(--space-xs);
    padding:       var(--space-xs) var(--space-sm);
    border-bottom: 1px solid var(--border);
    flex-shrink:   0;
  }

  .pp-name {
    flex:          1;
    font-size:     var(--font-size-sm);
    font-weight:   600;
    color:         var(--text);
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
  }

  .pp-close {
    background:  none;
    border:      none;
    cursor:      pointer;
    color:       var(--text-muted);
    font-size:   var(--font-size-base);
    padding:     var(--space-xs);
    line-height: 1;
    flex-shrink: 0;
  }
  .pp-close:hover { color: var(--text); }

  /* Media body */
  .pp-body {
    flex:       1;
    overflow:   hidden;
    display:    flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
    min-height: 0;
  }

  .pp-img-wrap {
    width:   100%;
    height:  100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-sm);
  }

  .pp-image {
    max-width:  100%;
    max-height: 100%;
    object-fit: contain;
  }

  .pp-video {
    width:      100%;
    max-height: 100%;
  }

  .pp-audio-wrap {
    display:        flex;
    flex-direction: column;
    align-items:    center;
    gap:            var(--space-sm);
    padding:        var(--space-sm);
    width:          100%;
  }

  .pp-audio {
    width: 100%;
  }

  .pp-pdf {
    width:  100%;
    height: 100%;
    border: none;
  }

  .pp-generic {
    display:        flex;
    flex-direction: column;
    align-items:    center;
    justify-content: center;
    width:          100%;
    height:         100%;
  }

  .pp-big-icon {
    font-size: 4rem;
    line-height: 1;
  }

  /* Metadata footer */
  .pp-meta {
    flex-shrink:   0;
    padding:       var(--space-sm);
    border-top:    1px solid var(--border);
    overflow-y:    auto;
    max-height:    12rem;
  }

  .pp-meta-list {
    display:               grid;
    grid-template-columns: auto 1fr;
    gap:                   0.15rem var(--space-sm);
    margin:                0;
    font-size:             var(--font-size-xs);
  }

  dt {
    color:       var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    white-space: nowrap;
  }

  dd {
    margin:        0;
    color:         var(--text-dim);
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
  }

  .pp-meta-name {
    font-weight: 500;
    color:       var(--text);
  }

  .pp-tags {
    display:   flex;
    flex-wrap: wrap;
    gap:       var(--space-xs);
    margin-top: var(--space-xs);
  }
</style>
