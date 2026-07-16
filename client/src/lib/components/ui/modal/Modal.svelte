<script lang="ts">
	import { fade, fly } from 'svelte/transition';
	import { modal, type ModalProps } from './index.js';
	import Icon from '../../Icon.svelte';
	import { IconButton } from '../icon-button/index.js';

	let {
		title,
		size,
		footerAlign,
		onClose,
		onsubmit,
		headerExtra,
		footer,
		class: klass,
		children
	}: ModalProps = $props();

	const s = $derived(modal({ size, footerAlign }));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class={s.scrim()} onclick={onClose} transition:fade={{ duration: 180 }}>
	<!--
	  stopPropagation keeps a click inside the dialog from reaching the scrim's close
	  handler. svelte:element lets RenameModal keep a real <form> root for Enter-to-submit.
	-->
	<svelte:element
		this={onsubmit ? 'form' : 'div'}
		role="dialog"
		aria-modal="true"
		aria-label={title}
		tabindex="-1"
		class={s.root({ class: klass })}
		onclick={(e: MouseEvent) => e.stopPropagation()}
		{onsubmit}
		transition:fly={{ y: 16, duration: 200 }}
	>
		<div class={s.header()}>
			<h2 class={s.title()}>{title}</h2>
			{#if headerExtra}{@render headerExtra()}{/if}
			<IconButton size="sm" class="ml-auto" onclick={onClose} aria-label="Close">
				<Icon name="Close" size={14} />
			</IconButton>
		</div>

		<div class={s.body()}>
			{@render children()}
		</div>

		{#if footer}
			<div class={s.footer()}>
				{@render footer()}
			</div>
		{/if}
	</svelte:element>
</div>
