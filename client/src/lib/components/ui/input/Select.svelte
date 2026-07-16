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
				// Height/font come from size="sm" below — NOT from here. tv applies the
				// extended recipe's `size` after this `variant`, so anything set here that
				// `size` also sets would lose.
				filter: 'w-auto appearance-none py-0 pr-7 pl-[10px]'
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

	const effSize = $derived(size ?? (variant === 'filter' ? 'sm' : 'md'));
</script>

{#snippet field()}
	<select bind:value class={select({ size: effSize, variant, class: klass })} {...rest}>
		{@render children()}
	</select>
{/snippet}

{#if variant === 'filter'}
	<span class="relative inline-flex items-center">
		{@render field()}
		<!--
		  11px, not 10px: the original was a background-image at `right 10px center`, and
		  background-origin is the padding box — so it cleared the 1px border too. This svg
		  is positioned against the border box, so it has to add that pixel back.
		-->
		<svg
			class="pointer-events-none absolute right-[11px]"
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
