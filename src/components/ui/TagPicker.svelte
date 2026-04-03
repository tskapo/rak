<!-- TagPicker.svelte : popover d'édition des tags d'un fichier ou dossier.
     Affiché via clic droit → "Étiqueter..." ou depuis le PreviewPanel.
     Liste les tags existants (cochables) avec leur TagBadge.
     Toggle → setFileTags() immédiat. "Gérer…" ouvre TagManager.
     Ferme sur Escape ou clic en dehors. -->

<script lang="ts">
  import TagBadge   from './TagBadge.svelte'
  import TagManager from './TagManager.svelte'
  import { getAllTags, getFileTags, setFileTags } from '../../stores/tags.svelte'
  import { t }                                    from '$lib/i18n.svelte'
  import type { FileEntry }                       from '$lib/tauri'

  interface Props {
    entry:    FileEntry
    x?:       number
    y?:       number
    onclose?: () => void
  }

  const { entry, x = 0, y = 0, onclose }: Props = $props()

  // ── Tag state ─────────────────────────────────────────────────────────────
  let checkedIds  = $state<Set<number>>(new Set())
  let showManager = $state(false)

  $effect(() => {
    getFileTags(entry.path)
      .then(tags => { checkedIds = new Set(tags.map(t => t.id)) })
      .catch(() => {})
  })

  async function toggle(id: number) {
    const next = new Set(checkedIds)
    if (next.has(id)) next.delete(id)
    else              next.add(id)
    checkedIds = next
    await setFileTags(entry.path, [...next]).catch(console.error)
  }

  // ── Clamp popover to viewport ─────────────────────────────────────────────
  const PICKER_W = 208
  const PICKER_H = 320
  const clampedX = $derived(Math.min(x, window.innerWidth  - PICKER_W - 8))
  const clampedY = $derived(Math.min(y, window.innerHeight - PICKER_H - 8))

  // ── Close on Escape ───────────────────────────────────────────────────────
  $effect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') { e.preventDefault(); onclose?.() }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  })
</script>

<!-- Transparent backdrop to catch outside clicks -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="tp-backdrop" role="presentation" onclick={() => onclose?.()}></div>

<!-- Popover -->
<div
  class="tag-picker"
  style:left="{clampedX}px"
  style:top="{clampedY}px"
  role="dialog"
  aria-modal="true"
  aria-label={t('tags.title')}
>
  <p class="tp-title">{t('tags.title')}</p>

  <ul class="tp-list">
    {#each getAllTags() as tag (tag.id)}
      <li>
        <label class="tp-row">
          <input
            type="checkbox"
            checked={checkedIds.has(tag.id)}
            onchange={() => toggle(tag.id)}
          />
          <TagBadge {tag} />
        </label>
      </li>
    {/each}
  </ul>

  <hr class="tp-sep" />
  <button class="tp-manage" onclick={() => { showManager = true }}>
    {t('tags.manage')}
  </button>
</div>

{#if showManager}
  <TagManager onclose={() => { showManager = false }} />
{/if}

<style>
  .tp-backdrop {
    position: fixed;
    inset:    0;
    z-index:  200;
  }

  .tag-picker {
    position:      fixed;
    z-index:       201;
    background:    var(--bg-toolbar);
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    box-shadow:    0 6px 18px rgba(0,0,0,.22);
    width:         13rem;
    max-height:    20rem;
    display:       flex;
    flex-direction: column;
    overflow:      hidden;
  }

  .tp-title {
    margin:      0;
    padding:     var(--space-xs) var(--space-sm);
    font-size:   var(--font-size-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color:       var(--text-muted);
    flex-shrink: 0;
  }

  .tp-list {
    list-style:  none;
    margin:      0;
    padding:     0 0 var(--space-xs);
    overflow-y:  auto;
    flex:        1;
  }

  .tp-row {
    display:     flex;
    align-items: center;
    gap:         var(--space-sm);
    padding:     var(--space-xs) var(--space-sm);
    cursor:      pointer;
    transition:  background var(--transition-fast);
  }

  .tp-row:hover { background: var(--bg-hover); }

  .tp-row input[type="checkbox"] {
    flex-shrink:  0;
    accent-color: var(--accent);
    cursor:       pointer;
  }

  .tp-sep {
    margin:     0;
    border:     none;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .tp-manage {
    display:     block;
    width:       100%;
    padding:     var(--space-xs) var(--space-sm);
    background:  none;
    border:      none;
    text-align:  left;
    font-family: inherit;
    font-size:   var(--font-size-sm);
    color:       var(--accent);
    cursor:      pointer;
    flex-shrink: 0;
  }

  .tp-manage:hover { background: var(--bg-hover); }
</style>
