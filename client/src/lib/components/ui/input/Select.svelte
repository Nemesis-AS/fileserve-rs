<script lang="ts">
	import { input, type SelectProps } from './index.js';
	import { tv } from '../tv.js';

	const select = tv({
		extend: input,
		base: 'cursor-pointer',
		variants: {
			variant: {
				default: '',
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
