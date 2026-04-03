<!-- RecentList.svelte : liste des dossiers et fichiers récemment visités.
     Lit depuis invoke('get_recents') au montage.
     Utilisé dans SidebarPane et dans la route /recent. -->

<script lang="ts">
  import { getRecents } from '$lib/tauri'
  import { navigate }   from '../../stores/navigation.svelte'
  import type { FileEntry } from '$lib/tauri'

  interface Props {
    limit?:      number
    activePath?: string
  }

  const { limit = 10, activePath = '' }: Props = $props()

  let recents = $state<FileEntry[]>([])

  $effect(() => {
    getRecents(limit).then(r => { recents = r }).catch(() => {})
  })

  function open(entry: FileEntry) {
    if (entry.isDir) navigate(entry.path).catch(console.error)
  }

  function icon(entry: FileEntry): string {
    if (entry.isDir) return '📁'
    const m = entry.mimeType ?? ''
    if (m.startsWith('image/')) return '🖼'
    if (m.startsWith('video/')) return '🎬'
    if (m.startsWith('audio/')) return '🎵'
    if (m === 'application/pdf') return '📕'
    return '📄'
  }
</script>

<ul class="recent-list">
  {#each recents as entry (entry.path)}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
    <li
      class="rl-item"
      class:active={entry.path === activePath}
      role="option"
      aria-selected={entry.path === activePath}
      onclick={() => open(entry)}
    >
      <span class="rl-icon">{icon(entry)}</span>
      <span class="rl-name">{entry.name}</span>
    </li>
  {/each}
</ul>

<style>
  .recent-list {
    list-style: none;
    margin:     0;
    padding:    0;
  }

  .rl-item {
    display:       flex;
    align-items:   center;
    gap:           var(--space-xs);
    padding:       0.2rem var(--space-sm);
    cursor:        pointer;
    border-radius: var(--radius-sm);
    transition:    background var(--transition-fast);
    font-size:     var(--font-size-sm);
    color:         var(--text-dim);
  }

  .rl-item:hover  { background: var(--bg-hover); color: var(--text); }
  .rl-item.active { background: var(--bg-sel);   color: var(--text); }

  .rl-icon { font-size: 0.85rem; flex-shrink: 0; }

  .rl-name {
    flex:          1;
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
  }
</style>
