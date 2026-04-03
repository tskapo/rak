<!-- TagManager.svelte : interface CRUD complète pour les tags globaux.
     Accessible via "Gérer les tags…" dans TagPicker ou les préférences.
     Permet de créer un tag (nom + couleur), renommer, recolorer et supprimer.
     La suppression déclenche ON DELETE CASCADE côté SQLite → retire le tag de tous les fichiers. -->

<script lang="ts">
  import TagBadge from './TagBadge.svelte'
  import { getAllTags, createTag, updateTag, deleteTag } from '../../stores/tags.svelte'
  import { t } from '$lib/i18n.svelte'
  import type { Tag } from '$lib/tauri'

  interface Props {
    onclose?: () => void
  }

  const { onclose }: Props = $props()

  // ── Inline edit state ─────────────────────────────────────────────────────
  let editingId   = $state<number | null>(null)
  let editName    = $state('')
  let editColor   = $state('')

  function startEdit(tag: Tag) {
    editingId = tag.id
    editName  = tag.name
    editColor = tag.color
  }

  async function saveEdit() {
    if (editingId === null || !editName.trim()) return
    await updateTag(editingId, editName.trim(), editColor).catch(console.error)
    cancelEdit()
  }

  function cancelEdit() {
    editingId = null
    editName  = ''
    editColor = ''
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter')  { e.preventDefault(); saveEdit() }
    if (e.key === 'Escape') { e.preventDefault(); cancelEdit() }
  }

  // ── Delete confirmation ───────────────────────────────────────────────────
  let deleteConfirmId = $state<number | null>(null)

  async function confirmDelete(id: number) {
    await deleteTag(id).catch(console.error)
    deleteConfirmId = null
  }

  // ── Create form ───────────────────────────────────────────────────────────
  const DEFAULT_COLOR = '#6c8ebf'
  let newName  = $state('')
  let newColor = $state(DEFAULT_COLOR)
  let creating = $state(false)

  async function handleCreate() {
    if (!newName.trim()) return
    creating = true
    await createTag(newName.trim(), newColor).catch(console.error)
    newName  = ''
    newColor = DEFAULT_COLOR
    creating = false
  }

  function handleCreateKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); handleCreate() }
  }

  // ── Close on Escape ───────────────────────────────────────────────────────
  $effect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape' && editingId === null && deleteConfirmId === null) {
        e.preventDefault()
        onclose?.()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  })
</script>

<!-- Backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="tm-backdrop" role="presentation" onclick={() => onclose?.()}></div>

<!-- Modal -->
<div class="tag-manager" role="dialog" aria-modal="true" aria-label={t('tags.manager_title')}>
  <!-- Header -->
  <div class="tm-header">
    <span class="tm-title">{t('tags.manager_title')}</span>
    <button class="tm-close" onclick={() => onclose?.()}>{t('actions.close')}</button>
  </div>

  <!-- Tag list -->
  <ul class="tm-list">
    {#each getAllTags() as tag (tag.id)}
      <li class="tm-row">
        {#if editingId === tag.id}
          <!-- Edit mode -->
          <input
            type="color"
            class="tm-color-input"
            bind:value={editColor}
          />
          <input
            class="tm-name-input"
            type="text"
            bind:value={editName}
            onkeydown={handleEditKeydown}
            autocomplete="off"
          />
          <button class="tm-btn tm-save" onclick={saveEdit}>{t('actions.save')}</button>
          <button class="tm-btn tm-cancel" onclick={cancelEdit}>{t('actions.cancel')}</button>

        {:else if deleteConfirmId === tag.id}
          <!-- Delete confirmation -->
          <span class="tm-confirm-msg">Delete "{tag.name}"?</span>
          <button class="tm-btn tm-danger" onclick={() => confirmDelete(tag.id)}>{t('actions.ok')}</button>
          <button class="tm-btn tm-cancel" onclick={() => { deleteConfirmId = null }}>{t('actions.cancel')}</button>

        {:else}
          <!-- Normal row -->
          <TagBadge {tag} />
          <span class="tm-spacer"></span>
          <button class="tm-btn tm-edit" onclick={() => startEdit(tag)}>✎</button>
          <button class="tm-btn tm-delete" onclick={() => { deleteConfirmId = tag.id }}>✕</button>
        {/if}
      </li>
    {/each}
  </ul>

  <!-- Create form -->
  <div class="tm-create">
    <input
      type="color"
      class="tm-color-input"
      bind:value={newColor}
    />
    <input
      class="tm-name-input"
      type="text"
      placeholder={t('tags.new_placeholder')}
      bind:value={newName}
      onkeydown={handleCreateKeydown}
      autocomplete="off"
    />
    <button
      class="tm-btn tm-create-btn"
      onclick={handleCreate}
      disabled={creating || !newName.trim()}
    >+</button>
  </div>
</div>

<style>
  .tm-backdrop {
    position: fixed;
    inset:    0;
    z-index:  300;
    background: rgba(0,0,0,.35);
  }

  .tag-manager {
    position:       fixed;
    z-index:        301;
    top:            50%;
    left:           50%;
    transform:      translate(-50%, -50%);
    width:          min(22rem, 90vw);
    max-height:     80vh;
    background:     var(--bg-toolbar);
    border:         1px solid var(--border);
    border-radius:  var(--radius-sm);
    box-shadow:     0 12px 32px rgba(0,0,0,.3);
    display:        flex;
    flex-direction: column;
    overflow:       hidden;
  }

  /* Header */
  .tm-header {
    display:       flex;
    align-items:   center;
    padding:       var(--space-sm) var(--space-sm) var(--space-sm) var(--space-md, var(--space-sm));
    border-bottom: 1px solid var(--border);
    flex-shrink:   0;
  }

  .tm-title {
    flex:        1;
    font-size:   var(--font-size-base);
    font-weight: 600;
    color:       var(--text);
  }

  .tm-close {
    background:  none;
    border:      none;
    cursor:      pointer;
    font-family: inherit;
    font-size:   var(--font-size-sm);
    color:       var(--text-muted);
    padding:     var(--space-xs);
  }
  .tm-close:hover { color: var(--text); }

  /* Tag list */
  .tm-list {
    list-style:  none;
    margin:      0;
    padding:     var(--space-xs) 0;
    overflow-y:  auto;
    flex:        1;
  }

  .tm-row {
    display:     flex;
    align-items: center;
    gap:         var(--space-xs);
    padding:     var(--space-xs) var(--space-sm);
    min-height:  2.25rem;
  }

  .tm-spacer { flex: 1; }

  /* Shared button style */
  .tm-btn {
    background:    none;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor:        pointer;
    font-family:   inherit;
    font-size:     var(--font-size-sm);
    color:         var(--text-dim);
    padding:       0.1rem var(--space-xs);
    white-space:   nowrap;
    transition:    background var(--transition-fast), color var(--transition-fast);
  }

  .tm-btn:hover         { background: var(--bg-hover); color: var(--text); }
  .tm-save:hover        { border-color: var(--accent); color: var(--accent); }
  .tm-danger            { color: var(--danger, #e53e3e); border-color: var(--danger, #e53e3e); }
  .tm-danger:hover      { background: var(--danger, #e53e3e); color: #fff; }
  .tm-create-btn        { border-color: var(--accent); color: var(--accent); }
  .tm-create-btn:hover  { background: var(--accent); color: #fff; }
  .tm-create-btn:disabled { opacity: .4; cursor: default; }

  .tm-confirm-msg {
    flex:       1;
    font-size:  var(--font-size-sm);
    color:      var(--text);
  }

  /* Inline inputs */
  .tm-color-input {
    width:         1.75rem;
    height:        1.75rem;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    padding:       0;
    cursor:        pointer;
    flex-shrink:   0;
    background:    none;
  }

  .tm-name-input {
    flex:          1;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    background:    var(--bg);
    color:         var(--text);
    font-family:   inherit;
    font-size:     var(--font-size-base);
    padding:       0.2rem var(--space-xs);
    outline:       none;
  }

  .tm-name-input:focus { border-color: var(--accent); }

  /* Create form */
  .tm-create {
    display:       flex;
    align-items:   center;
    gap:           var(--space-xs);
    padding:       var(--space-xs) var(--space-sm);
    border-top:    1px solid var(--border);
    flex-shrink:   0;
  }
</style>
