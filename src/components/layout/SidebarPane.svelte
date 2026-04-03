<!-- SidebarPane.svelte : panneau latéral gauche.
     Sections : Périphériques (volumes montés + disk labels), Favoris, Récents.
     Arborescence de dossiers collapsible (TreeNode) sous chaque volume.
     Clic droit sur un volume → édition inline du label (nom + couleur). -->

<script lang="ts">
  import RecentList from '../ui/RecentList.svelte'
  import TreeNode   from './TreeNode.svelte'
  import { listVolumes, type FileEntry, type DiskLabel } from '$lib/tauri'
  import { loadDiskLabel, getDiskLabelFor, saveDiskLabelFor } from '../../stores/diskLabels.svelte'
  import { getFavorites, addFavorite, removeFavorite } from '../../stores/appPrefs.svelte'
  import { getCurrentPath, navigate } from '../../stores/navigation.svelte'
  import { t } from '$lib/i18n.svelte'

  // ── Volumes ───────────────────────────────────────────────────────────────
  let volumes = $state<FileEntry[]>([])

  $effect(() => {
    listVolumes().then(vols => {
      volumes = vols
      for (const v of vols) loadDiskLabel(v.path).catch(() => {})
    }).catch(() => {})
  })

  // ── Disk label inline editor ───────────────────────────────────────────────
  let editingVolume  = $state<string | null>(null)
  let editLabelName  = $state('')
  let editLabelColor = $state('#6c8ebf')

  function startEditLabel(vol: FileEntry) {
    const existing = getDiskLabelFor(vol.path)
    editingVolume  = vol.path
    editLabelName  = existing?.customName ?? vol.name
    editLabelColor = existing?.color      ?? '#6c8ebf'
  }

  async function saveLabelEdit() {
    if (!editingVolume) return
    const label: DiskLabel = {
      path:       editingVolume,
      color:      editLabelColor,
      customName: editLabelName.trim() || undefined,
    }
    await saveDiskLabelFor(label).catch(console.error)
    editingVolume = null
  }

  function cancelLabelEdit() { editingVolume = null }

  function handleLabelKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter')  { e.preventDefault(); saveLabelEdit() }
    if (e.key === 'Escape') { e.preventDefault(); cancelLabelEdit() }
  }

  // ── Favorites ─────────────────────────────────────────────────────────────
  function pinCurrent() {
    const p = getCurrentPath()
    if (p) addFavorite(p).catch(console.error)
  }

  // ── Section collapse ──────────────────────────────────────────────────────
  let showDevices   = $state(true)
  let showFavorites = $state(true)
  let showRecents   = $state(true)
</script>

<aside class="sidebar">

  <!-- ── Devices ──────────────────────────────────────────────────────────── -->
  <section class="sb-section">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="sb-section-header" role="button" tabindex="-1" onclick={() => showDevices = !showDevices}>
      <span class="sb-section-title">{t('sidebar.devices')}</span>
      <span class="sb-chevron">{showDevices ? '▾' : '▸'}</span>
    </div>

    {#if showDevices}
      <ul class="sb-list">
        {#each volumes as vol (vol.path)}
          {@const label = getDiskLabelFor(vol.path)}
          <li>
            {#if editingVolume === vol.path}
              <!-- Inline label editor -->
              <div class="sb-label-edit">
                <input type="color" class="sb-color-input" bind:value={editLabelColor} />
                <input
                  class="sb-name-input"
                  type="text"
                  bind:value={editLabelName}
                  onkeydown={handleLabelKeydown}
                  autocomplete="off"
                />
                <button class="sb-btn" onclick={saveLabelEdit}>{t('actions.save')}</button>
                <button class="sb-btn" onclick={cancelLabelEdit}>{t('actions.cancel')}</button>
              </div>
            {:else}
              <!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
              <div
                class="sb-item sb-volume"
                class:active={getCurrentPath().startsWith(vol.path)}
                role="button"
                tabindex="-1"
                onclick={() => navigate(vol.path).catch(console.error)}
                oncontextmenu={(e) => { e.preventDefault(); startEditLabel(vol) }}
              >
                <span
                  class="sb-dot"
                  style:background={label?.color ?? '#888'}
                ></span>
                <span class="sb-label">{label?.customName ?? vol.name}</span>
              </div>

              <!-- Collapsible tree under this volume -->
              <ul class="sb-tree">
                <TreeNode entry={vol} depth={0} />
              </ul>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- ── Favorites ─────────────────────────────────────────────────────────── -->
  <section class="sb-section">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="sb-section-header" role="button" tabindex="-1" onclick={() => showFavorites = !showFavorites}>
      <span class="sb-section-title">{t('sidebar.favorites')}</span>
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <span class="sb-pin" role="button" tabindex="-1" onclick={(e) => { e.stopPropagation(); pinCurrent() }}>＋</span>
      <span class="sb-chevron">{showFavorites ? '▾' : '▸'}</span>
    </div>

    {#if showFavorites}
      <ul class="sb-list">
        {#each getFavorites() as path (path)}
          {@const name = path.split('/').filter(Boolean).at(-1) ?? path}
          <!-- svelte-ignore a11y_interactive_supports_focus -->
          <li
            class="sb-item"
            class:active={getCurrentPath() === path}
            role="option"
            aria-selected={getCurrentPath() === path}
            onclick={() => navigate(path).catch(console.error)}
            onkeydown={(e) => { if (e.key === 'Enter') navigate(path).catch(console.error) }}
            oncontextmenu={(e) => { e.preventDefault(); removeFavorite(path).catch(console.error) }}
          >
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <span class="sb-fav-icon">⭐</span>
            <span class="sb-label">{name}</span>
          </li>
        {:else}
          <li class="sb-empty">Right-click folder to remove</li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- ── Recents ───────────────────────────────────────────────────────────── -->
  <section class="sb-section">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="sb-section-header" role="button" tabindex="-1" onclick={() => showRecents = !showRecents}>
      <span class="sb-section-title">{t('sidebar.recents')}</span>
      <span class="sb-chevron">{showRecents ? '▾' : '▸'}</span>
    </div>

    {#if showRecents}
      <RecentList limit={8} activePath={getCurrentPath()} />
    {/if}
  </section>

</aside>

<style>
  .sidebar {
    width:          13rem;
    flex-shrink:    0;
    height:         100%;
    overflow-y:     auto;
    background:     var(--bg-sidebar, var(--bg));
    border-right:   1px solid var(--border);
    display:        flex;
    flex-direction: column;
    padding:        var(--space-xs) 0;
  }

  /* Section */
  .sb-section {
    margin-bottom: var(--space-xs);
  }

  .sb-section-header {
    display:     flex;
    align-items: center;
    padding:     0.15rem var(--space-sm);
    cursor:      pointer;
    user-select: none;
  }

  .sb-section-title {
    flex:           1;
    font-size:      var(--font-size-xs);
    font-weight:    700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color:          var(--text-muted);
  }

  .sb-chevron {
    font-size:  0.55rem;
    color:      var(--text-muted);
    margin-left: var(--space-xs);
  }

  .sb-pin {
    font-size:  0.85rem;
    color:      var(--text-muted);
    padding:    0 var(--space-xs);
    line-height: 1;
  }
  .sb-pin:hover { color: var(--accent); }

  /* Item list */
  .sb-list {
    list-style: none;
    margin:     0;
    padding:    0;
  }

  .sb-item {
    display:       flex;
    align-items:   center;
    gap:           var(--space-xs);
    padding:       0.2rem var(--space-sm);
    cursor:        pointer;
    border-radius: var(--radius-sm);
    font-size:     var(--font-size-sm);
    color:         var(--text-dim);
    transition:    background var(--transition-fast);
  }

  .sb-item:hover  { background: var(--bg-hover); color: var(--text); }
  .sb-item.active { background: var(--bg-sel);   color: var(--text); font-weight: 600; }

  /* Volume color dot */
  .sb-dot {
    width:        0.55rem;
    height:       0.55rem;
    border-radius: 50%;
    flex-shrink:  0;
  }

  .sb-fav-icon {
    font-size:   0.8rem;
    flex-shrink: 0;
  }

  .sb-label {
    flex:          1;
    overflow:      hidden;
    text-overflow: ellipsis;
    white-space:   nowrap;
  }

  .sb-empty {
    padding:   var(--space-xs) var(--space-sm);
    font-size: var(--font-size-xs);
    color:     var(--text-muted);
    font-style: italic;
  }

  /* Folder tree */
  .sb-tree {
    margin:     0;
    padding:    0;
    list-style: none;
  }

  /* Inline label editor */
  .sb-label-edit {
    display:    flex;
    align-items: center;
    gap:        var(--space-xs);
    padding:    var(--space-xs) var(--space-sm);
  }

  .sb-color-input {
    width:         1.5rem;
    height:        1.5rem;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    padding:       0;
    cursor:        pointer;
    background:    none;
    flex-shrink:   0;
  }

  .sb-name-input {
    flex:          1;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    background:    var(--bg);
    color:         var(--text);
    font-family:   inherit;
    font-size:     var(--font-size-sm);
    padding:       0.15rem var(--space-xs);
    outline:       none;
    min-width:     0;
  }
  .sb-name-input:focus { border-color: var(--accent); }

  .sb-btn {
    background:    none;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor:        pointer;
    font-family:   inherit;
    font-size:     var(--font-size-xs);
    color:         var(--text-dim);
    padding:       0.1rem var(--space-xs);
    white-space:   nowrap;
    flex-shrink:   0;
  }
  .sb-btn:hover { background: var(--bg-hover); }
</style>
