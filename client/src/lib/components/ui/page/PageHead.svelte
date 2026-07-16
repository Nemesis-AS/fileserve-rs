<script lang="ts">
	import type { Snippet } from 'svelte';
	import { tv } from '../tv.js';

	/**
	 * Port of `.page__head` — title, optional sub-copy, and trailing actions.
	 * `.page__head .btn { margin-left: auto }` becomes an explicit `ml-auto` on the
	 * actions wrapper rather than a descendant selector reaching into children.
	 */
	const head = tv({
		slots: {
			root: 'mb-[18px] flex flex-wrap items-baseline gap-3',
			title: 'm-0 text-[19px] font-semibold tracking-[-0.01em]',
			sub: 'm-0 text-[13px] text-ink-muted',
			actions: 'ml-auto flex items-center gap-2'
		}
	});

	let {
		title,
		sub,
		class: klass,
		actions,
		children
	}: {
		title: string;
		sub?: string;
		class?: string;
		actions?: Snippet;
		children?: Snippet;
	} = $props();

	const s = head();
</script>

<div class={s.root({ class: klass })}>
	<h1 class={s.title()}>{title}</h1>
	{#if sub}<p class={s.sub()}>{sub}</p>{/if}
	{#if children}{@render children()}{/if}
	{#if actions}
		<div class={s.actions()}>{@render actions()}</div>
	{/if}
</div>
