<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { tv, type WithClass } from '../tv.js';

	/** Port of `.check` — the label wrapper and its `accent-color`-tinted box. */
	const check = tv({ base: 'flex cursor-pointer items-center gap-2 text-[13px]' });

	let {
		checked = $bindable(false),
		class: klass,
		children,
		...rest
	}: WithClass<HTMLInputAttributes> & { checked?: boolean; children: Snippet } = $props();
</script>

<label class={check({ class: klass })}>
	<input type="checkbox" bind:checked class="accent-accent" {...rest} />
	{@render children()}
</label>
