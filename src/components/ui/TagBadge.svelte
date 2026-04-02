<!-- TagBadge.svelte : pill coloré affichant le nom d'un tag.
     Props : tag (Tag), removable? (boolean).
     Callback onremove si removable=true et l'utilisateur clique la croix.
     Couleur de fond = tag.color avec opacité réduite ; texte en couleur pleine. -->

<script lang="ts">
  import type { Tag } from '$lib/tauri'

  interface Props {
    tag:        Tag
    removable?: boolean
    onremove?:  (tag: Tag) => void
  }

  const { tag, removable = false, onremove }: Props = $props()

  /** Convert hex color to rgba with reduced opacity for the background. */
  function hexToRgba(hex: string, alpha: number): string {
    const r = parseInt(hex.slice(1, 3), 16)
    const g = parseInt(hex.slice(3, 5), 16)
    const b = parseInt(hex.slice(5, 7), 16)
    return `rgba(${r}, ${g}, ${b}, ${alpha})`
  }
</script>

<span
  class="tag-badge"
  style:background={hexToRgba(tag.color, 0.18)}
  style:color={tag.color}
  style:border-color={hexToRgba(tag.color, 0.35)}
>
  {tag.name}
  {#if removable}
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button
      class="remove-btn"
      onclick={(e) => { e.stopPropagation(); onremove?.(tag) }}
      tabindex="-1"
    >×</button>
  {/if}
</span>

<style>
  .tag-badge {
    display:       inline-flex;
    align-items:   center;
    gap:           0.2rem;
    font-size:     var(--font-size-xs);
    font-weight:   600;
    padding:       0.1rem 0.4rem;
    border-radius: var(--radius-pill);
    border:        1px solid transparent;
    white-space:   nowrap;
    line-height:   1.4;
  }

  .remove-btn {
    background:    none;
    border:        none;
    padding:       0;
    cursor:        pointer;
    color:         inherit;
    font-size:     0.75rem;
    line-height:   1;
    opacity:       0.7;
    font-family:   inherit;
  }

  .remove-btn:hover { opacity: 1; }
</style>
