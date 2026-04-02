<script lang="ts" generics="T">
// VirtualList.svelte : composant de scroll virtuel à hauteur fixe par item.
// Props : items[] (données), itemHeight (px), containerHeight (px).
// Snippet : children({ item, index }) — rendu délégué au parent.
// Ne rend que les items visibles + BUFFER items au-dessus et en-dessous.

import type { Snippet } from 'svelte'

interface Props {
	items:           T[]
	/** Height of each row in pixels. Compute from rem in the parent when ui_scale changes:
	 *  `parseFloat(getComputedStyle(document.documentElement).fontSize) * ROW_HEIGHT_REM` */
	itemHeight?:     number
	containerHeight: number
	children:        Snippet<[{ item: T; index: number }]>
}

/** Extra rows rendered above and below the visible window to avoid blank flicker on fast scroll. */
const BUFFER = 3

let {
	items,
	itemHeight = 40,
	containerHeight,
	children,
}: Props = $props()

let scrollTop = $state(0)

/** Total scrollable height — sets the inner div so the scrollbar is correct. */
const totalHeight = $derived(items.length * itemHeight)

/** Index of the first rendered item (clamped to 0). */
const startIndex = $derived(
	Math.max(0, Math.floor(scrollTop / itemHeight) - BUFFER)
)

/** Index one past the last rendered item (clamped to items.length). */
const endIndex = $derived(
	Math.min(
	items.length,
	Math.ceil((scrollTop + containerHeight) / itemHeight) + BUFFER
	)
)

const visibleItems = $derived(items.slice(startIndex, endIndex))

/** Vertical offset of the rendered window inside the scrollable container. */
const offsetY = $derived(startIndex * itemHeight)

function handleScroll(e: Event) {
	scrollTop = (e.currentTarget as HTMLElement).scrollTop
}
</script>

<!--
Outer div: the actual scrollable viewport.
Inner div: full-height spacer so the scrollbar reflects total item count.
Window div: absolutely positioned, translated to the correct offset.
-->
<div
class="virtual-scroll"
style:height="{containerHeight}px"
onscroll={handleScroll}
role="list"
>
<div class="virtual-inner" style:height="{totalHeight}px">
	<div class="virtual-window" style:transform="translateY({offsetY}px)">
	{#each visibleItems as item, i (startIndex + i)}
		<div class="virtual-item" style:height="{itemHeight}px" role="listitem">
		{@render children({ item, index: startIndex + i })}
		</div>
	{/each}
	</div>
</div>
</div>

<style>
.virtual-scroll {
	overflow-y: auto;
	overflow-x: hidden;
	position: relative;
	width: 100%;
}

.virtual-inner {
	position: relative;
	width: 100%;
}

.virtual-window {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
}

.virtual-item {
	overflow: hidden;
}
</style>
