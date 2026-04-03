<!-- SidebarPane.svelte : panneau latéral gauche.
     Sections : Périphériques (volumes montés + disk labels), Favoris, Récents.
     Arborescence de dossiers collapsible (TreeNode) sous chaque volume.
     Clic droit sur un volume → édition inline du label (nom + couleur). -->

<script lang="ts">
  import RecentList        from '../ui/RecentList.svelte'
  import TreeNode          from './TreeNode.svelte'
  import DiskLabelPopover  from '../ui/DiskLabelPopover.svelte'
  import { listVolumes, type FileEntry } from '$lib/tauri'
  import { loadDiskLabel, getDiskLabelFor, saveDiskLabelFor } from '../../stores/diskLabels.svelte'
  import { getFavorites, addFavorite, removeFavorite } from '../../stores/appPrefs.svelte'
  import { getCurrentPath, navigate } from '../../stores/navigation.svelte'
  import { t }       from '$lib/i18n.svelte'
  import LangPicker  from '../ui/LangPicker.svelte'
  import { goto }    from '$app/navigation'
  import { getPrefs, setUiScale } from '../../stores/appPrefs.svelte'

  // ── Volumes ───────────────────────────────────────────────────────────────
  let volumes = $state<FileEntry[]>([])

  $effect(() => {
    listVolumes().then(vols => {
      volumes = vols
      for (const v of vols) loadDiskLabel(v.path).catch(() => {})
    }).catch(() => {})
  })

  // ── Disk label popover ─────────────────────────────────────────────────────
  let popoverVol = $state<FileEntry | null>(null)
  let popoverX   = $state(0)
  let popoverY   = $state(0)

  function openDiskPopover(vol: FileEntry, e: MouseEvent) {
    popoverVol = vol
    popoverX   = e.clientX
    popoverY   = e.clientY
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
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
            <div
              class="sb-item sb-volume"
              class:active={getCurrentPath().startsWith(vol.path)}
              role="button"
              tabindex="-1"
              onclick={() => navigate(vol.path).catch(console.error)}
              ondblclick={(e) => { e.preventDefault(); openDiskPopover(vol, e) }}
            >
              <span class="sb-dot" style:background={label?.color ?? '#888'}></span>
              <span class="sb-label">{label?.customName ?? vol.name}</span>
            </div>

            <!-- Collapsible tree under this volume -->
            <ul class="sb-tree">
              <TreeNode entry={vol} depth={0} />
            </ul>
          </li>
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
      <button class="sb-more-btn" onclick={() => goto('/recent')}>
        {t('sidebar.recents')} →
      </button>
    {/if}
  </section>

  <!-- ── Footer: scale slider + language picker ───────────────────────────── -->
  <div class="sb-footer">
    <div class="sb-scale">
      <span class="sb-scale-label">A</span>
      <input
        type="range"
        min="0.8"
        max="1.4"
        step="0.05"
        value={getPrefs()?.uiScale ?? 1}
        oninput={(e) => setUiScale(parseFloat((e.currentTarget as HTMLInputElement).value))}
        class="sb-scale-slider"
        title="UI scale"
      />
      <span class="sb-scale-label sb-scale-lg">A</span>
    </div>
    <LangPicker />
  </div>

  {#if popoverVol}
    <DiskLabelPopover
      path={popoverVol.path}
      initialColor={getDiskLabelFor(popoverVol.path)?.color}
      initialName={getDiskLabelFor(popoverVol.path)?.customName ?? popoverVol.name}
      x={popoverX}
      y={popoverY}
      onclose={() => popoverVol = null}
    />
  {/if}
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

  .sb-more-btn {
    display:     block;
    width:       100%;
    background:  none;
    border:      none;
    text-align:  left;
    font-family: inherit;
    font-size:   var(--font-size-xs);
    color:       var(--accent);
    padding:     0.1rem var(--space-sm);
    cursor:      pointer;
  }
  .sb-more-btn:hover { text-decoration: underline; }

  /* Footer */
  .sb-footer {
    margin-top:     auto;
    padding:        var(--space-xs) var(--space-sm);
    border-top:     1px solid var(--border);
    flex-shrink:    0;
    display:        flex;
    flex-direction: column;
    gap:            var(--space-xs);
  }

  /* Scale slider */
  .sb-scale {
    display:     flex;
    align-items: center;
    gap:         var(--space-xs);
  }

  .sb-scale-label {
    font-size:   0.65rem;
    color:       var(--text-muted);
    flex-shrink: 0;
    user-select: none;
  }

  .sb-scale-lg { font-size: 1rem; }

  .sb-scale-slider {
    flex:         1;
    accent-color: var(--accent);
    cursor:       pointer;
  }
</style>
