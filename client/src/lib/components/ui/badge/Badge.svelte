<script lang="ts">
	import { badge, type BadgeProps } from './index.js';

	/**
	 * `dot` is opt-in rather than implied by tone: today the dot is explicit `<i>` markup,
	 * and the audit log renders a `suspended` badge *without* one (it wants the colour only).
	 */
	let {
		tone,
		dot = false,
		class: klass,
		children,
		...rest
	}: BadgeProps & { dot?: boolean } = $props();
</script>

<span class={badge({ tone, class: klass })} {...rest}>
	{#if dot}
		<i class="inline-block size-1.5 rounded-full bg-current"></i>
	{/if}
	{@render children()}
</span>
