<script lang="ts">
	import type { Snippet } from 'svelte';
	import { tv } from '../tv.js';

	/**
	 * Port of `.dropzone` / `.dropzone--active`. Hover and drag-active share one
	 * treatment, so `active` just forces the hover look.
	 */
	const dropzone = tv({
		base: 'cursor-pointer rounded-[10px] border-[1.5px] border-dashed border-edge-strong bg-elevated px-5 py-7 text-center text-ink-muted transition-[border-color,background-color,color] duration-150 hover:border-accent hover:bg-accent-soft hover:text-accent-ink [&_svg]:mb-2 [&_svg]:size-7',
		variants: {
			active: { true: 'border-accent bg-accent-soft text-accent-ink', false: '' }
		},
		defaultVariants: { active: false }
	});

	let {
		active = false,
		hint,
		class: klass,
		children,
		...rest
	}: {
		active?: boolean;
		hint?: string;
		class?: string;
		children: Snippet;
		[key: string]: unknown;
	} = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class={dropzone({ active, class: klass })} {...rest}>
	{@render children()}
	{#if hint}<div class="mt-1 text-[12px]">{hint}</div>{/if}
</div>
