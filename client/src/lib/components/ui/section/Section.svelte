<script lang="ts">
	import type { Snippet } from 'svelte';
	import { tv } from '../tv.js';

	/**
	 * `.settings-section` and `.form-section` were byte-for-byte identical (border,
	 * radius 10px, 16px padding, uppercase eyebrow label, 12px-gap body) — one component.
	 */
	const section = tv({
		slots: {
			root: 'rounded-[10px] border border-edge bg-surface p-4',
			label: 'mb-3 text-[11px] font-semibold tracking-[0.06em] text-ink-faint uppercase',
			body: 'flex flex-col gap-3'
		}
	});

	let {
		label,
		class: klass,
		children
	}: { label?: string; class?: string; children: Snippet } = $props();

	const s = section();
</script>

<div class={s.root({ class: klass })}>
	{#if label}<div class={s.label()}>{label}</div>{/if}
	<div class={s.body()}>
		{@render children()}
	</div>
</div>
