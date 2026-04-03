<!-- TagFilterPopover.svelte : popover flottant de filtrage par tags.
     Affiche tous les tags disponibles sous forme de cases à cocher.
     Props : activeTagIds (ids actifs), x/y (position), onchange, onclose. -->

<script lang="ts">
  import { getAllTags } from '../../stores/tags.svelte'
  import { t } from '$lib/i18n.svelte'

  interface Props {
    activeTagIds: number[]
    x:            number
    y:            number
    onchange:     (ids: number[]) => void
    onclose:      () => void
  }

  const { activeTagIds, x, y, onchange, onclose }: Props = $props()

  const tags = $derived(getAllTags())

  function toggle(id: number) {
    if (activeTagIds.includes(id)) {
      onchange(activeTagIds.filter(i => i !== id))
    } else {
      onchange([...activeTagIds, id])
    }
  }

  // Clamp position to viewport
  const LEFT = $derived(Math.min(x, window.innerWidth  - 180))
  const TOP  = $derived(Math.min(y, window.innerHeight - 300))
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="tfp-backdrop" role="presentation" onclick={onclose}></div>

<div class="tfp-menu" style:left="{LEFT}px" style:top="{TOP}px">
  {#if tags.length === 0}
    <p class="tfp-empty">{t('tags.no_tags')}</p>
  {:else}
    {#each tags as tag (tag.id)}
      <label class="tfp-row">
        <input
          type="checkbox"
          checked={activeTagIds.includes(tag.id)}
          onchange={() => toggle(tag.id)}
        />
        <span class="tfp-swatch" style:background={tag.color}></span>
        <span class="tfp-name">{tag.name}</span>
      </label>
    {/each}
  {/if}

  {#if activeTagIds.length > 0}
    <div class="tfp-footer">
      <button class="tfp-clear" onclick={() => onchange([])}>{t('tags.clear_filter')}</button>
    </div>
  {/if}
</div>

<style>
  .tfp-backdrop {
    position: fixed;
    inset:    0;
    z-index:  100;
  }

  .tfp-menu {
    position:      fixed;
    z-index:       101;
    min-width:     10rem;
    max-height:    18rem;
    overflow-y:    auto;
    background:    var(--bg-toolbar);
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    box-shadow:    0 4px 12px rgba(0,0,0,.18);
    padding:       var(--space-xs) 0;
  }

  .tfp-row {
    display:     flex;
    align-items: center;
    gap:         var(--space-xs);
    padding:     0.25rem var(--space-sm);
    cursor:      pointer;
    font-size:   var(--font-size-base);
    color:       var(--text);
  }

  .tfp-row:hover { background: var(--bg-hover); }

  .tfp-swatch {
    width:         0.625rem;
    height:        0.625rem;
    border-radius: 50%;
    flex-shrink:   0;
  }

  .tfp-name {
    flex: 1;
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
  }

  .tfp-empty {
    padding:    var(--space-xs) var(--space-sm);
    color:      var(--text-muted);
    font-size:  var(--font-size-sm);
    font-style: italic;
    margin:     0;
  }

  .tfp-footer {
    border-top: 1px solid var(--border);
    margin-top: var(--space-xs);
    padding:    var(--space-xs) var(--space-sm) 0;
  }

  .tfp-clear {
    background:    none;
    border:        none;
    cursor:        pointer;
    font-family:   inherit;
    font-size:     var(--font-size-sm);
    color:         var(--accent);
    padding:       0;
  }

  .tfp-clear:hover { text-decoration: underline; }
</style>
