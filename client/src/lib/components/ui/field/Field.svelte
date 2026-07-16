<script lang="ts">
	import type { FieldProps } from './index.js';
	import { tv } from '../tv.js';
	import Hint from './Hint.svelte';

	/** Port of `.field` + its `label` + `.field-hint`. */
	const field = tv({ base: 'mb-[14px] flex flex-col gap-1.5' });

	let { label, hint, for: htmlFor, class: klass, children }: FieldProps = $props();
</script>

<div class={field({ class: klass })}>
	{#if label}
		<label for={htmlFor} class="text-[12px] font-medium text-ink">{label}</label>
	{/if}
	{@render children()}
	{#if hint}
		<Hint>{hint}</Hint>
	{/if}
</div>
