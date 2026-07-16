<script lang="ts">
	import { fly } from 'svelte/transition';
	import { menu, type MenuProps } from './index.js';

	/**
	 * The fly transition lives on the popover itself (as it did in TopBar/GridTile) —
	 * putting it on an inner wrapper would pop the surface in instantly and only slide
	 * its contents. The two call sites used slightly different distances, hence the props.
	 *
	 * Positioning is the caller's job: the topbar anchors under its trigger, the grid tile
	 * under its action bar. Both parents are `relative`.
	 */
	let { size, class: klass, flyY = -6, flyDuration = 150, children }: MenuProps = $props();
</script>

<div class={menu({ size, class: klass })} transition:fly={{ y: flyY, duration: flyDuration }}>
	{@render children()}
</div>
