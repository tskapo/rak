<!-- +page.svelte : route racine /.
     Redirige immédiatement vers /browser via goto() de SvelteKit. -->

<script lang="ts">
  import { goto } from '$app/navigation'
  import { onMount } from 'svelte'

  onMount(() => { goto('/browser', { replaceState: true }) })
</script>
