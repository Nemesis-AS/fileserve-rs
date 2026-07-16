<script lang="ts">
	import type { Snippet } from 'svelte';

	/** Port of `.menu__header` — a bold title line over muted detail. */
	let { title, children }: { title: string; children?: Snippet } = $props();
</script>

<div class="px-2.5 pt-[9px] pb-2 text-[12px] text-ink-muted">
	<b class="block text-[13px] font-medium text-ink">{title}</b>
	{#if children}{@render children()}{/if}
</div>
