<script lang="ts">
	import type { Snippet } from 'svelte';
	import { tv } from '../tv.js';

	/** Port of `.empty` / `__inner` / `__art` and its h3 + p. */
	const empty = tv({
		slots: {
			root: 'grid flex-1 place-items-center px-6 py-[60px]',
			inner: 'flex max-w-[360px] flex-col items-center gap-1.5 text-center',
			art: 'mb-2 grid size-16 place-items-center rounded-[14px] bg-sunken text-ink-faint [&_svg]:size-7',
			title: 'm-0 text-[15px] font-semibold',
			body: 'mt-0 mr-0 mb-3 ml-0 text-[13px] text-ink-muted'
		}
	});

	let {
		title,
		body,
		art,
		class: klass,
		children
	}: {
		title: string;
		body?: string;
		/** The illustration — an icon, rendered inside the rounded plate. */
		art?: Snippet;
		class?: string;
		/** Trailing actions, e.g. an Upload button. */
		children?: Snippet;
	} = $props();

	const s = empty();
</script>

<div class={s.root({ class: klass })}>
	<div class={s.inner()}>
		{#if art}<div class={s.art()}>{@render art()}</div>{/if}
		<h3 class={s.title()}>{title}</h3>
		{#if body}<p class={s.body()}>{body}</p>{/if}
		{#if children}{@render children()}{/if}
	</div>
</div>
