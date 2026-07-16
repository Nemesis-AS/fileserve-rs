<script lang="ts">
	import type { Snippet } from 'svelte';
	import { tv } from '../tv.js';

	/** A plain bordered surface. Used by the design showcase and settings-style panels. */
	const card = tv({
		base: 'overflow-hidden rounded-[10px] border border-edge bg-surface',
		variants: {
			padded: { true: 'px-5 py-[18px]', false: '' },
			split: { true: 'grid grid-cols-2', false: '' }
		},
		defaultVariants: { padded: true, split: false }
	});

	let {
		padded,
		split,
		class: klass,
		children
	}: { padded?: boolean; split?: boolean; class?: string; children: Snippet } = $props();
</script>

<div class={card({ padded: split ? false : padded, split, class: klass })}>
	{@render children()}
</div>
