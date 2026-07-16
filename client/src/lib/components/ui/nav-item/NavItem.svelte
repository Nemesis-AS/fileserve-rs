<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { tv, type WithClass } from '../tv.js';

	/**
	 * Port of `.nav-item` + `[data-active='1']` + `__icon` + `__count`.
	 * The icon recolours with the active state, hence `group` + `group-data-`.
	 */
	const navItem = tv({
		slots: {
			root: 'group flex w-full cursor-pointer items-center gap-[9px] rounded-[7px] border-0 bg-transparent px-2.5 py-1.5 text-left text-[13.5px] whitespace-nowrap text-ink transition-colors duration-100 select-none hover:bg-row-hover data-[active=1]:bg-row-active data-[active=1]:font-medium data-[active=1]:text-accent-ink',
			icon: 'size-4 shrink-0 text-ink-muted group-data-[active=1]:text-accent-ink',
			count: 'ml-auto text-[11.5px] text-ink-faint tabular-nums'
		}
	});

	let {
		active = false,
		count,
		icon,
		class: klass,
		children,
		...rest
	}: WithClass<HTMLButtonAttributes> & {
		active?: boolean;
		count?: number;
		icon: Snippet;
		children: Snippet;
	} = $props();

	const s = navItem();
</script>

<button type="button" class={s.root({ class: klass })} data-active={active ? '1' : '0'} {...rest}>
	<span class={s.icon()}>{@render icon()}</span>
	{@render children()}
	{#if count !== undefined}<span class={s.count()}>{count}</span>{/if}
</button>
