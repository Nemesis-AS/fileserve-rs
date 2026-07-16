<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { tv, type WithClass } from '../tv.js';
	import Icon from '../../Icon.svelte';

	/**
	 * Port of `.search` — a sunken field that lifts to a bordered surface on focus.
	 * The `kbd` hint is optional: the topbar shows `/`, the audit filter shows nothing.
	 */
	const search = tv({
		slots: {
			root: 'flex h-[30px] items-center gap-1.5 rounded-[7px] border border-transparent bg-sunken px-2.5 text-ink-muted transition-[border-color,background-color] duration-150 focus-within:border-edge-strong focus-within:bg-surface focus-within:text-ink',
			field: 'min-w-0 flex-1 border-0 bg-transparent font-[inherit] text-inherit outline-0',
			hint: 'shrink-0 rounded border border-edge bg-surface px-[5px] py-px font-code text-[10.5px] text-ink-faint'
		}
	});

	let {
		value = $bindable(''),
		kbd,
		class: klass,
		ref = $bindable(),
		...rest
	}: WithClass<HTMLInputAttributes> & {
		value?: string;
		kbd?: string;
		ref?: HTMLInputElement | null;
	} = $props();

	const s = search();
</script>

<label class={s.root({ class: klass })}>
	<Icon name="Search" size={14} class="shrink-0" />
	<input bind:this={ref} type="text" bind:value class={s.field()} {...rest} />
	{#if kbd}<kbd class={s.hint()}>{kbd}</kbd>{/if}
</label>
