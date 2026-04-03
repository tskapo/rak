<!-- TreeNode.svelte : nœud récursif de l'arborescence dans la sidebar.
     Affiche un dossier avec expand/collapse. Les enfants sont chargés à la
     première expansion via listDir(). -->

<script lang="ts">
  import { listDir }  from '$lib/tauri'
  import { navigate, getCurrentPath } from '../../stores/navigation.svelte'
  import type { FileEntry } from '$lib/tauri'
  import TreeNode from './TreeNode.svelte'

  interface Props {
    entry: FileEntry
    depth?: number
  }

  const { entry, depth = 0 }: Props = $props()

  let expanded = $state(false)
  let children = $state<FileEntry[]>([])
  let loaded   = $state(false)

  async function toggle() {
    if (!expanded && !loaded) {
      const result = await listDir(entry.path).catch(() => ({ entries: [] }))
      children = result.entries
        .filter(e => e.isDir)
        .sort((a, b) => a.name.localeCompare(b.name))
      loaded = true
    }
    expanded = !expanded
  }

  function open() {
    navigate(entry.path).catch(console.error)
  }

  const isActive = $derived(getCurrentPath() === entry.path)
  const isAncestor = $derived(getCurrentPath().startsWith(entry.path + '/'))
</script>

<li class="tree-node">
  <div
    class="tn-row"
    class:active={isActive}
    class:ancestor={isAncestor}
    style:padding-left="{0.5 + depth * 0.9}rem"
  >
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <span
      class="tn-toggle"
      onclick={toggle}
      role="button"
      tabindex="-1"
      aria-label={expanded ? 'Collapse' : 'Expand'}
    >{expanded ? '▾' : '▸'}</span>

    <!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
    <span class="tn-label" role="button" tabindex="-1" onclick={open}>
      📁 {entry.name}
    </span>
  </div>

  {#if expanded && children.length > 0}
    <ul class="tn-children">
      {#each children as child (child.path)}
        <TreeNode entry={child} depth={depth + 1} />
      {/each}
    </ul>
  {/if}
</li>

<style>
  .tree-node { list-style: none; }

  .tn-row {
    display:       flex;
    align-items:   center;
    gap:           0.2rem;
    height:        1.75rem;
    cursor:        pointer;
    border-radius: var(--radius-sm);
    transition:    background var(--transition-fast);
  }

  .tn-row:hover    { background: var(--bg-hover); }
  .tn-row.active   { background: var(--bg-sel); }
  .tn-row.ancestor { background: var(--bg-hover); }

  .tn-toggle {
    width:      1rem;
    flex-shrink: 0;
    font-size:  0.6rem;
    color:      var(--text-muted);
    text-align: center;
    padding:    0 0.1rem;
    user-select: none;
  }

  .tn-label {
    flex:          1;
    font-size:     var(--font-size-sm);
    color:         var(--text-dim);
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
  }

  .tn-row.active   .tn-label { color: var(--text); font-weight: 600; }
  .tn-row.ancestor .tn-label { color: var(--text); }

  .tn-children {
    margin:  0;
    padding: 0;
  }
</style>
