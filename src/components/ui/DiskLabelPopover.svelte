<!-- DiskLabelPopover.svelte : popover flottant d'édition de l'étiquette d'un disque.
     Déclenché par double-clic sur un volume dans la sidebar.
     Palette de 10 couleurs preset + champ texte pour le surnom + OK / Annuler. -->

<script lang="ts">
  import { saveDiskLabelFor } from '../../stores/diskLabels.svelte'
  import { t } from '$lib/i18n.svelte'

  interface Props {
    path:          string
    initialColor?: string
    initialName?:  string
    x:             number
    y:             number
    onclose:       () => void
  }

  const {
    path,
    initialColor = '#4a90d9',
    initialName  = '',
    x,
    y,
    onclose,
  }: Props = $props()

  const COLORS = [
    '#4a90d9', '#e8a838', '#6bbf6b', '#e55555', '#c77dff',
    '#ff7f50', '#20b2aa', '#fa8072', '#888888', '#f0c040',
  ]

  const POPOVER_W = 220

  let selectedColor = $state(initialColor)
  let nameValue     = $state(initialName)

  // Keep popover inside the viewport
  const clampedX = $derived(
    Math.min(x, (typeof window !== 'undefined' ? window.innerWidth : 800) - POPOVER_W - 8)
  )

  async function save() {
    await saveDiskLabelFor({
      path,
      color:      selectedColor,
      customName: nameValue.trim() || undefined,
    }).catch(console.error)
    onclose()
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.preventDefault(); onclose() }
    if (e.key === 'Enter')  { e.preventDefault(); save() }
  }

  // Close when clicking outside
  $effect(() => {
    const onMousedown = (e: MouseEvent) => {
      if (!(e.target as Element).closest('.disk-label-popover')) onclose()
    }
    window.addEventListener('mousedown', onMousedown)
    return () => window.removeEventListener('mousedown', onMousedown)
  })
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="disk-label-popover"
  style:left="{clampedX}px"
  style:top="{y}px"
  role="dialog"
  onkeydown={handleKeydown}
>
  <div class="dlp-title">{t('disk.label.title')}</div>

  <div class="dlp-colors">
    {#each COLORS as color (color)}
      <button
        class="dlp-swatch"
        class:selected={selectedColor === color}
        style:background={color}
        onclick={() => selectedColor = color}
        aria-label={color}
      ></button>
    {/each}
  </div>

  <input
    class="dlp-name"
    type="text"
    bind:value={nameValue}
    placeholder={t('disk.label.name_placeholder')}
    onkeydown={handleKeydown}
    autocomplete="off"
  />

  <div class="dlp-actions">
    <button class="dlp-btn" onclick={onclose}>{t('actions.cancel')}</button>
    <button class="dlp-btn dlp-btn-primary" onclick={save}>{t('actions.ok')}</button>
  </div>
</div>

<style>
  .disk-label-popover {
    position:       fixed;
    z-index:        500;
    background:     var(--bg-popover);
    border:         1px solid var(--border);
    border-radius:  var(--radius-lg);
    padding:        var(--space-md);
    width:          13.75rem;
    box-shadow:     var(--shadow-lg);
    display:        flex;
    flex-direction: column;
    gap:            var(--space-sm);
  }

  .dlp-title {
    font-size:      var(--font-size-xs);
    font-weight:    700;
    color:          var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .dlp-colors {
    display:   flex;
    flex-wrap: wrap;
    gap:       var(--space-xs);
  }

  .dlp-swatch {
    width:         1.375rem;
    height:        1.375rem;
    border-radius: 50%;
    border:        2px solid transparent;
    cursor:        pointer;
    padding:       0;
    transition:    transform var(--transition-fast), border-color var(--transition-fast);
  }
  .dlp-swatch:hover    { transform: scale(1.15); }
  .dlp-swatch.selected { border-color: var(--text); }

  .dlp-name {
    width:         100%;
    background:    var(--bg-hover);
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    padding:       0.3rem var(--space-sm);
    color:         var(--text);
    font-family:   inherit;
    font-size:     var(--font-size-sm);
    outline:       none;
  }
  .dlp-name:focus { border-color: var(--accent); }

  .dlp-actions {
    display:         flex;
    justify-content: flex-end;
    gap:             var(--space-xs);
  }

  .dlp-btn {
    height:        1.5rem;
    padding:       0 var(--space-sm);
    border-radius: var(--radius-sm);
    font-size:     var(--font-size-sm);
    font-family:   inherit;
    cursor:        pointer;
    border:        1px solid var(--border);
    background:    transparent;
    color:         var(--text-dim);
    transition:    all var(--transition-fast);
  }
  .dlp-btn:hover         { color: var(--text); }
  .dlp-btn-primary       { background: var(--accent); color: #fff; border-color: var(--accent); }
  .dlp-btn-primary:hover { opacity: 0.85; }
</style>
