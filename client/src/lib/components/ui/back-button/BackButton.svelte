<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { tv, type WithClass } from '../tv.js';
	import Icon from '../../Icon.svelte';

	/**
	 * Port of `.viewer__back`. Despite the name it was never viewer-specific — the two
	 * admin user forms used it too, so it lives in ui/ rather than with the viewer.
	 */
	const backButton = tv({
		base: 'flex cursor-pointer items-center gap-1 rounded-md border-0 bg-transparent py-1 pr-2 pl-1 text-[13px] whitespace-nowrap text-ink-muted transition-[background-color,color] duration-100 hover:bg-row-hover hover:text-ink [&_svg]:size-[14px]'
	});

	let {
		class: klass,
		children,
		...rest
	}: WithClass<HTMLButtonAttributes> & { children: Snippet } = $props();
</script>

<button type="button" class={backButton({ class: klass })} {...rest}>
	<Icon name="ChevronL" size={14} />
	{@render children()}
</button>
