<script lang="ts">
	import { input, type SelectProps } from './index.js';
	import { tv } from '../tv.js';

	/**
	 * `variant="filter"` is the old `.audit-filters__select`: auto width, smaller text,
	 * and a custom arrow. That arrow was a 10x6 *triangle* baked in as a data-URI
	 * background (layout.css `.audit-filters__select`), not a chevron — it is redrawn
	 * inline here so the shape stays identical while the data-URI goes away.
	 * The default variant keeps the native OS arrow, as it does today.
	 */
	const select = tv({
		extend: input,
		base: 'cursor-pointer',
		variants: {
			variant: {
				default: '',
				filter: 'h-[30px] w-auto appearance-none py-0 pr-7 pl-[10px] text-[12.5px]'
			}
		},
		defaultVariants: { variant: 'default' }
	});

	let {
		size,
		variant = 'default',
		class: klass,
		value = $bindable(),
		children,
		...rest
	}: SelectProps & { variant?: 'default' | 'filter' } = $props();
</script>

{#snippet field()}
	<select bind:value class={select({ size, variant, class: klass })} {...rest}>
		{@render children()}
	</select>
{/snippet}

{#if variant === 'filter'}
	<span class="relative inline-flex items-center">
		{@render field()}
		<svg
			class="pointer-events-none absolute right-[10px]"
			width="10"
			height="6"
			viewBox="0 0 10 6"
			aria-hidden="true"
		>
			<path fill="#999" d="M0 0h10L5 6z" />
		</svg>
	</span>
{:else}
	{@render field()}
{/if}
