<!-- AppShell.svelte : mise en page principale de l'application.
     Compose SidebarPane (gauche) et le contenu principal (BrowserView ou autre route).
     Gère la résolution du thème via themeStore ($effect sur data-theme). -->

<script lang="ts">
  import SidebarPane from './SidebarPane.svelte'
  import type { Snippet } from 'svelte'

  interface Props {
    children: Snippet
  }

  const { children }: Props = $props()
</script>

<div class="app-shell">
  <SidebarPane />
  <main class="app-main">
    {@render children()}
  </main>
</div>

<style>
  .app-shell {
    display:    flex;
    height:     100vh;
    overflow:   hidden;
    background: var(--bg);
  }

  .app-main {
    flex:     1;
    display:  flex;
    overflow: hidden;
    min-width: 0;
  }
</style>
