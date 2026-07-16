<script lang="ts" generics="T extends string">
	import type { Snippet } from 'svelte';
	import { tv } from '../tv.js';

	/**
	 * Port of `.view-toggle` — a sunken pill where the active button lifts onto a raised
	 * surface. Icon-only today (list/grid), so items render via a snippet.
	 *
	 * Not shared with `.settings__tabs`: those are a vertical text list with an accent
	 * wash, not a segmented control. They look nothing alike beyond both being tab-ish.
	 */
	const segmented = tv({
		slots: {
			root: 'flex rounded-[7px] bg-sunken p-0.5',
			item: 'grid h-[22px] w-[26px] cursor-pointer place-items-center rounded-[5px] border-0 bg-transparent text-ink-muted transition-[background-color,box-shadow] duration-100 data-[active=1]:bg-surface data-[active=1]:text-ink data-[active=1]:shadow-xs [&_svg]:size-[13px]'
		}
	});

	let {
		value,
		options,
		onchange,
		class: klass,
		item
	}: {
		value: T;
		options: { value: T; label: string }[];
		onchange: (v: T) => void;
		class?: string;
		/** Renders the contents of each button (an icon). */
		item: Snippet<[T]>;
	} = $props();

	const s = segmented();
</script>

<div class={s.root({ class: klass })}>
	{#each options as opt (opt.value)}
		<button
			type="button"
			class={s.item()}
			data-active={value === opt.value ? '1' : '0'}
			onclick={() => onchange(opt.value)}
			title={opt.label}
			aria-label={opt.label}
		>
			{@render item(opt.value)}
		</button>
	{/each}
</div>
