<script lang="ts">
	import type { Snippet } from 'svelte';
	import { tv } from '../tv.js';

	/**
	 * Port of `.page` — the scrolling content column under the topbar.
	 * `scroll-area` (layout.css) supplies the custom scrollbar; Tailwind has no
	 * equivalent for ::-webkit-scrollbar.
	 */
	const page = tv({ base: 'scroll-area min-h-0 flex-1 overflow-auto p-page' });

	let { class: klass, children }: { class?: string; children: Snippet } = $props();
</script>

<div class={page({ class: klass })}>
	{@render children()}
</div>
